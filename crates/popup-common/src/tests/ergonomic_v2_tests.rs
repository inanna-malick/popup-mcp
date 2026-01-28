use crate::Element;
use serde_json::json;

#[test]
fn test_ergonomic_macro_usage() {
    // This test simulates how a user (or LLM) would write the JSON using the json! macro
    // and verifies it deserializes correctly into the strict Element enum.

    let json_input = json!({
        "title": "Ergonomic Test",
        "elements": [
            // 1. Implicit ID + String Options
            {
                "select": "Choose Flavor",
                // "id": "choose_flavor" (Auto-generated)
                "options": "Vanilla, Chocolate, Strawberry"
            },
            // 2. Implicit Text Child + String Options
            {
                "multi": "Toppings",
                "options": "Sprinkles, Nuts, Sauce",
                "Sauce": "Warning: Sauce is messy." // -> {"text": "Warning..."}
            },
            // 3. Single Object Child
            {
                "check": "Add Spoon",
                "reveals": { // -> [{"input": "Spoon Type"}]
                    "input": "Spoon Type",
                    "placeholder": "Plastic or Metal?"
                }
            }
        ]
    });

    // We need to parse the full definition, but Element is internal to the crate.
    // So we test Element deserialization directly.
    let elements_json = json_input["elements"].clone();
    let elements: Vec<Element> = serde_json::from_value(elements_json).expect("Failed to deserialize ergonomic elements");

    assert_eq!(elements.len(), 3);

    // Verify Element 1: Select
    match &elements[0] {
        Element::Select { select, id, options, .. } => {
            assert_eq!(select, "Choose Flavor");
            assert_eq!(id, "choose_flavor");
            assert_eq!(options.len(), 3);
            assert_eq!(options[0].value(), "Vanilla");
        }
        _ => panic!("Element 0 should be Select"),
    }

    // Verify Element 2: Multi with Implicit Text Child
    match &elements[1] {
        Element::Multi { multi, option_children, .. } => {
            assert_eq!(multi, "Toppings");
            assert!(option_children.contains_key("Sauce"));
            let children = &option_children["Sauce"];
            assert_eq!(children.len(), 1);
            match &children[0] {
                Element::Text { text, .. } => assert_eq!(text, "Warning: Sauce is messy."),
                _ => panic!("Child of Sauce should be Text"),
            }
        }
        _ => panic!("Element 1 should be Multi"),
    }

    // Verify Element 3: Check with Single Object Reveal
    match &elements[2] {
        Element::Check { check, reveals, .. } => {
            assert_eq!(check, "Add Spoon");
            assert_eq!(reveals.len(), 1);
            match &reveals[0] {
                Element::Input { input, .. } => assert_eq!(input, "Spoon Type"),
                _ => panic!("Reveal should be Input"),
            }
        }
        _ => panic!("Element 2 should be Check"),
    }
}
