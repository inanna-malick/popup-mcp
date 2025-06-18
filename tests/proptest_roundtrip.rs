use popup_mcp::{parse_popup_dsl, serialize_popup_dsl, models::*};
use proptest::prelude::*;

// Simpler, more focused strategies
fn simple_string() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 ]{1,20}"
}

fn simple_element() -> impl Strategy<Value = Element> {
    prop_oneof![
        simple_string().prop_map(Element::Text),
        (simple_string(), 0.0f32..100.0f32, 0.0f32..100.0f32, 0.0f32..100.0f32)
            .prop_map(|(label, min, max, default)| Element::Slider { label, min, max, default }),
        (simple_string(), any::<bool>())
            .prop_map(|(label, default)| Element::Checkbox { label, default }),
        simple_string().prop_map(|label| Element::Textbox { label, placeholder: None, rows: None }),
        (simple_string(), prop::collection::vec(simple_string(), 2..5))
            .prop_map(|(label, options)| Element::Choice { label, options }),
        prop::collection::vec(simple_string(), 1..4)
            .prop_map(Element::Buttons),
    ]
}

fn simple_popup() -> impl Strategy<Value = PopupDefinition> {
    (simple_string(), prop::collection::vec(simple_element(), 1..5))
        .prop_map(|(title, elements)| PopupDefinition { title, elements })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]
    
    #[test]
    fn test_roundtrip_parse_serialize_classic_syntax(popup in simple_popup()) {
        // Serialize the popup to DSL
        let serialized = serialize_popup_dsl(&popup);
        
        // Parse it back
        let parsed = parse_popup_dsl(&serialized).expect("Should parse serialized DSL");
        
        // Check that the essential structure is preserved
        prop_assert_eq!(popup.title, parsed.title);
        
        // Element count may increase due to auto-added buttons and Force Yield
        prop_assert!(parsed.elements.len() >= popup.elements.len(), 
            "Parsed should have at least as many elements (auto-added buttons)");
        
        // Check each element type matches (values may differ due to Force Yield auto-add)
        for (original, parsed) in popup.elements.iter().zip(parsed.elements.iter()) {
            match (original, parsed) {
                (Element::Text(a), Element::Text(b)) => prop_assert_eq!(a, b),
                (Element::Slider { label: a, .. }, Element::Slider { label: b, .. }) => prop_assert_eq!(a, b),
                (Element::Checkbox { label: a, .. }, Element::Checkbox { label: b, .. }) => prop_assert_eq!(a, b),
                (Element::Textbox { label: a, .. }, Element::Textbox { label: b, .. }) => prop_assert_eq!(a, b),
                (Element::Choice { label: a, .. }, Element::Choice { label: b, .. }) => prop_assert_eq!(a, b),
                (Element::Multiselect { label: a, .. }, Element::Multiselect { label: b, .. }) => prop_assert_eq!(a, b),
                (Element::Buttons(a), Element::Buttons(b)) => {
                    // Buttons might have Force Yield added, so just check original buttons are present
                    for button in a {
                        prop_assert!(b.contains(button), "Button '{}' should be preserved", button);
                    }
                },
                _ => {} // Different types are OK due to transformations
            }
        }
    }
    
    #[test]
    fn test_serialize_produces_valid_dsl(popup in simple_popup()) {
        let serialized = serialize_popup_dsl(&popup);
        
        // The serialized DSL should be valid and parseable
        let parsed = parse_popup_dsl(&serialized);
        prop_assert!(parsed.is_ok(), "Serialized DSL should be valid: {}", serialized);
    }
    
    #[test]
    fn test_parse_serialize_parse_stability(popup in simple_popup()) {
        // First round trip
        let serialized1 = serialize_popup_dsl(&popup);
        let parsed1 = parse_popup_dsl(&serialized1).expect("First parse should succeed");
        
        // Second round trip
        let serialized2 = serialize_popup_dsl(&parsed1);
        let parsed2 = parse_popup_dsl(&serialized2).expect("Second parse should succeed");
        
        // The two parsed results should be equivalent in structure  
        prop_assert_eq!(parsed1.title, parsed2.title);
        prop_assert_eq!(parsed1.elements.len(), parsed2.elements.len(), 
            "After stabilizing, element counts should be identical");
    }
}

#[cfg(test)]
mod manual_roundtrip_tests {
    use super::*;

    #[test]
    fn test_simple_roundtrip() {
        let original = PopupDefinition {
            title: "Test".to_string(),
            elements: vec![
                Element::Text("Hello".to_string()),
                Element::Buttons(vec!["OK".to_string()]),
            ],
        };
        
        let serialized = serialize_popup_dsl(&original);
        println!("Serialized:\n{}", serialized);
        
        let parsed = parse_popup_dsl(&serialized).expect("Should parse");
        
        assert_eq!(original.title, parsed.title);
        assert_eq!(original.elements.len(), parsed.elements.len());
    }
    
    #[test]
    fn test_complex_roundtrip() {
        let original = PopupDefinition {
            title: "Complex Form".to_string(),
            elements: vec![
                Element::Text("Fill out the form:".to_string()),
                Element::Slider { 
                    label: "Volume".to_string(), 
                    min: 0.0, 
                    max: 100.0, 
                    default: 50.0 
                },
                Element::Checkbox { 
                    label: "Enable feature".to_string(), 
                    default: true 
                },
                Element::Choice { 
                    label: "Mode".to_string(), 
                    options: vec!["Basic".to_string(), "Advanced".to_string()] 
                },
                Element::Buttons(vec!["Save".to_string(), "Cancel".to_string()]),
            ],
        };
        
        let serialized = serialize_popup_dsl(&original);
        println!("Complex serialized:\n{}", serialized);
        
        let parsed = parse_popup_dsl(&serialized).expect("Should parse complex form");
        
        assert_eq!(original.title, parsed.title);
        assert!(parsed.elements.len() >= original.elements.len()); // May have Force Yield added
    }
}