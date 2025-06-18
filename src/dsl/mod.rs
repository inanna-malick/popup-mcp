use anyhow::Result;
use pest::Parser;
use pest_derive::Parser;

use crate::models::{Element, PopupDefinition, Condition, ComparisonOp};

#[derive(Parser)]
#[grammar = "popup.pest"]
pub struct PopupParser;

pub fn parse_popup_dsl(input: &str) -> Result<PopupDefinition> {
    let pairs = PopupParser::parse(Rule::popup, input)
        .map_err(|e| {
            // Pest errors include detailed position and expected tokens
            anyhow::anyhow!("Parse error: {}", e)
        })?;
    
    let pair = pairs
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No popup found"))?;
    
    let mut definition = parse_popup(pair)?;
    
    // Ensure popup has buttons and Force Yield safety
    ensure_button_safety(&mut definition);
    
    Ok(definition)
}

fn parse_popup(pair: pest::iterators::Pair<Rule>) -> Result<PopupDefinition> {
    let mut inner = pair.into_inner();
    let first = inner.next().unwrap();
    
    match first.as_rule() {
        Rule::classic_popup => parse_classic_popup(first),
        Rule::simplified_popup => parse_simplified_popup(first),
        _ => Err(anyhow::anyhow!("Unexpected popup format"))
    }
}

fn parse_classic_popup(pair: pest::iterators::Pair<Rule>) -> Result<PopupDefinition> {
    let mut inner = pair.into_inner();
    
    // Get title from quoted string
    let title = parse_string(inner.next().unwrap())?;
    
    // Parse elements
    let mut elements = Vec::new();
    for pair in inner {
        if pair.as_rule() == Rule::EOI {
            continue;
        }
        if let Ok(element) = parse_element(pair) {
            elements.push(element);
        }
    }
    
    Ok(PopupDefinition { title, elements })
}

fn parse_simplified_popup(pair: pest::iterators::Pair<Rule>) -> Result<PopupDefinition> {
    let mut inner = pair.into_inner();
    
    // Get title from unquoted text before colon
    let title = inner.next().unwrap().as_str().trim().to_string();
    
    // Parse elements
    let mut elements = Vec::new();
    for pair in inner {
        if pair.as_rule() == Rule::EOI {
            continue;
        }
        if let Ok(element) = parse_element(pair) {
            elements.push(element);
        }
    }
    
    Ok(PopupDefinition { title, elements })
}

fn parse_element(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let inner_pair = if pair.as_rule() == Rule::element {
        // If it's an element wrapper, get the actual element inside
        pair.into_inner().next().unwrap()
    } else {
        // Otherwise use the pair directly
        pair
    };
    
    match inner_pair.as_rule() {
        Rule::text => parse_text(inner_pair),
        Rule::slider => parse_slider(inner_pair),
        Rule::checkbox => parse_checkbox(inner_pair),
        Rule::textbox => parse_textbox(inner_pair),
        Rule::choice => parse_choice(inner_pair),
        Rule::multiselect => parse_multiselect(inner_pair),
        Rule::group => parse_group(inner_pair),
        Rule::buttons => parse_buttons(inner_pair),
        Rule::conditional => parse_conditional(inner_pair),
        Rule::bare_text => parse_bare_text(inner_pair),
        Rule::EOI => Err(anyhow::anyhow!("EOI is not an element")),
        _ => Err(anyhow::anyhow!("Unknown element type: {:?}", inner_pair.as_rule()))
    }
}

// Macro to reduce parsing repetition
macro_rules! parse_with_label {
    ($pair:expr, $variant:ident) => {{
        let mut inner = $pair.into_inner();
        let label = parse_string(inner.next().unwrap())?;
        let options = parse_string_list_rule(inner.next().unwrap())?;
        Ok(Element::$variant { label, options })
    }};
}

fn parse_text(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let text = parse_string(pair.into_inner().next().unwrap())?;
    Ok(Element::Text(text))
}

fn parse_slider(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let mut inner = pair.into_inner();
    let label = parse_string(inner.next().unwrap())?;
    let min = inner.next().unwrap().as_str().parse::<f32>()?;
    let max = inner.next().unwrap().as_str().parse::<f32>()?;
    let default = inner.next()
        .map(|p| p.as_str().parse::<f32>())
        .transpose()?
        .unwrap_or((min + max) / 2.0);
    
    Ok(Element::Slider { label, min, max, default })
}

fn parse_checkbox(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let mut inner = pair.into_inner();
    let label = parse_string(inner.next().unwrap())?;
    let default = inner.next()
        .map(|p| p.as_str() == "true")
        .unwrap_or(false);
    
    Ok(Element::Checkbox { label, default })
}

fn parse_textbox(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let mut inner = pair.into_inner();
    let label = parse_string(inner.next().unwrap())?;
    let mut placeholder = None;
    let mut rows = None;
    
    for pair in inner {
        match pair.as_rule() {
            Rule::string => placeholder = Some(parse_string(pair)?),
            Rule::number => rows = Some(pair.as_str().parse::<u32>()?),
            _ => {}
        }
    }
    
    Ok(Element::Textbox { label, placeholder, rows })
}

fn parse_choice(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    parse_with_label!(pair, Choice)
}

fn parse_multiselect(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    parse_with_label!(pair, Multiselect)
}

fn parse_group(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let mut inner = pair.into_inner();
    let label = parse_string(inner.next().unwrap())?;
    let elements = parse_elements(inner)?;
    Ok(Element::Group { label, elements })
}

fn parse_buttons(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let string_list_pair = pair.into_inner().next().unwrap();
    let buttons = parse_string_list_rule(string_list_pair)?;
    Ok(Element::Buttons(buttons))
}

fn parse_conditional(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let mut inner = pair.into_inner();
    let condition_pair = inner.next().unwrap();
    let condition = parse_condition(condition_pair)?;
    let elements = parse_elements(inner)?;
    Ok(Element::Conditional { condition, elements })
}

// Helper functions to reduce duplication
fn parse_string_list_rule(pair: pest::iterators::Pair<Rule>) -> Result<Vec<String>> {
    let mut strings = Vec::new();
    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::string {
            strings.push(parse_string(inner_pair)?);
        }
    }
    Ok(strings)
}

fn parse_elements(pairs: pest::iterators::Pairs<Rule>) -> Result<Vec<Element>> {
    let mut elements = Vec::new();
    for pair in pairs {
        if let Ok(element) = parse_element(pair) {
            elements.push(element);
        }
    }
    Ok(elements)
}

fn parse_string(pair: pest::iterators::Pair<Rule>) -> Result<String> {
    // The string rule now includes quotes, so we need to strip them
    let full_string = pair.as_str();
    // Remove the surrounding quotes
    let content = &full_string[1..full_string.len()-1];
    Ok(content.to_string())
}

fn parse_condition(pair: pest::iterators::Pair<Rule>) -> Result<Condition> {
    let condition_str = pair.as_str();
    let mut inner = pair.into_inner();
    
    // The condition rule now contains the entire condition string
    // We need to determine which type based on the pattern
    if condition_str.starts_with("checked(") {
        // checked("name")
        let name = parse_string(inner.next().unwrap())?;
        Ok(Condition::Checked(name))
    } else if condition_str.starts_with("selected(") {
        // selected("name", "value")
        let name = parse_string(inner.next().unwrap())?;
        let value = parse_string(inner.next().unwrap())?;
        Ok(Condition::Selected(name, value))
    } else if condition_str.starts_with("count(") {
        // count("name") op value
        let name = parse_string(inner.next().unwrap())?;
        let op_pair = inner.next().unwrap();
        let op = parse_op(op_pair)?;
        let value = inner.next().unwrap().as_str().parse::<i32>()?;
        Ok(Condition::Count(name, op, value))
    } else {
        Err(anyhow::anyhow!("Unknown condition type: {}", condition_str))
    }
}

fn parse_op(pair: pest::iterators::Pair<Rule>) -> Result<ComparisonOp> {
    match pair.as_str() {
        ">" => Ok(ComparisonOp::Greater),
        "<" => Ok(ComparisonOp::Less),
        ">=" => Ok(ComparisonOp::GreaterEqual),
        "<=" => Ok(ComparisonOp::LessEqual),
        "==" => Ok(ComparisonOp::Equal),
        "!=" => Err(anyhow::anyhow!("!= operator not yet supported in conditions")),
        _ => Err(anyhow::anyhow!("Unknown comparison operator: {}", pair.as_str()))
    }
}

/// Ensures popup safety: adds buttons if missing and includes Force Yield
fn ensure_button_safety(definition: &mut PopupDefinition) {
    // Add default buttons if none exist
    if !has_buttons_recursive(&definition.elements) {
        eprintln!(
            "Warning: Popup '{}' had no buttons defined. Adding default 'Continue' button.",
            definition.title
        );
        definition.elements.push(Element::Buttons(vec!["Continue".to_string()]));
    }
    
    // Add Force Yield to the last button element
    add_force_yield_to_last_buttons(&mut definition.elements);
}

fn has_buttons_recursive(elements: &[Element]) -> bool {
    elements.iter().any(|element| match element {
        Element::Buttons(_) => true,
        Element::Group { elements, .. } | Element::Conditional { elements, .. } => {
            has_buttons_recursive(elements)
        }
        _ => false,
    })
}

fn add_force_yield_to_last_buttons(elements: &mut [Element]) -> bool {
    for element in elements.iter_mut().rev() {
        match element {
            Element::Buttons(buttons) => {
                if !buttons.iter().any(|b| b == "Force Yield") {
                    buttons.push("Force Yield".to_string());
                }
                return true;
            }
            Element::Group { elements: nested, .. } => {
                if add_force_yield_to_last_buttons(nested) {
                    return true;
                }
            }
            Element::Conditional { .. } => continue, // Skip conditional buttons
            _ => {}
        }
    }
    false
}

fn parse_bare_text(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let text = pair.as_str().trim();
    
    // Check if this looks like "Label: [widget]" format
    if let Some(colon_pos) = text.find(':') {
        let label = text[..colon_pos].trim();
        let rest = text[colon_pos + 1..].trim();
        
        if rest.starts_with('[') && rest.ends_with(']') {
            let widget_type = &rest[1..rest.len() - 1].trim();
            
            match *widget_type {
                "text" => Ok(Element::Text(label.to_string())),
                "textbox" => Ok(Element::Textbox { 
                    label: label.to_string(), 
                    placeholder: None, 
                    rows: None 
                }),
                "checkbox" => Ok(Element::Checkbox { 
                    label: label.to_string(), 
                    default: false 
                }),
                "Y/N" => Ok(Element::Choice { 
                    label: label.to_string(), 
                    options: vec!["Y".to_string(), "N".to_string()] 
                }),
                _ => {
                    // Not a recognized widget type, treat as plain text
                    Ok(Element::Text(text.to_string()))
                }
            }
        } else {
            // Doesn't match [widget] pattern, treat as plain text
            Ok(Element::Text(text.to_string()))
        }
    } else {
        // No colon, treat as plain text
        Ok(Element::Text(text.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classic_popup_syntax() {
        let input = r#"popup "Test Title" [
            text "Hello World"
            buttons ["OK", "Cancel"]
        ]"#;
        
        let result = parse_popup_dsl(input);
        assert!(result.is_ok());
        
        let popup = result.unwrap();
        assert_eq!(popup.title, "Test Title");
        assert_eq!(popup.elements.len(), 2);
    }

    #[test]
    fn test_simplified_popup_syntax() {
        let input = r#"[Test Title:
            text "Hello World"
            buttons ["OK", "Cancel"]
        ]"#;
        
        let result = parse_popup_dsl(input);
        assert!(result.is_ok());
        
        let popup = result.unwrap();
        assert_eq!(popup.title, "Test Title");
        assert_eq!(popup.elements.len(), 2);
    }

    #[test]
    fn test_all_widget_types() {
        let input = r#"popup "Widget Test" [
            text "Display text"
            slider "Volume" 0..100 @50
            checkbox "Enable feature" @true
            textbox "Enter name"
            choice "Select option" ["A", "B", "C"]
            multiselect "Choose many" ["X", "Y", "Z"]
            group "Settings" [
                checkbox "Advanced" @false
            ]
            buttons ["Save", "Cancel"]
        ]"#;
        
        let result = parse_popup_dsl(input);
        assert!(result.is_ok());
        
        let popup = result.unwrap();
        assert_eq!(popup.elements.len(), 8);
    }

    #[test]
    fn test_simple_text_syntax() {
        let input = r#"[Quick Form:
            Name: [textbox]
            Agree: [checkbox]
            Ready: [Y/N]
            buttons ["Submit"]
        ]"#;
        
        let result = parse_popup_dsl(input);
        if let Err(e) = &result {
            eprintln!("Parse error in test_simple_text_syntax: {}", e);
        }
        assert!(result.is_ok());
        
        let popup = result.unwrap();
        assert_eq!(popup.title, "Quick Form");
        assert_eq!(popup.elements.len(), 4);
        
        // Check that simple text elements were parsed correctly
        match &popup.elements[0] {
            Element::Textbox { label, .. } => assert_eq!(label, "Name"),
            _ => panic!("Expected textbox"),
        }
        
        match &popup.elements[1] {
            Element::Checkbox { label, .. } => assert_eq!(label, "Agree"),
            _ => panic!("Expected checkbox"),
        }
        
        match &popup.elements[2] {
            Element::Choice { label, options } => {
                assert_eq!(label, "Ready");
                assert_eq!(options, &["Y", "N"]);
            },
            _ => panic!("Expected choice"),
        }
    }

    #[test]
    fn test_conditional_elements() {
        let input = r#"popup "Conditional Test" [
            checkbox "Show advanced" @false
            if checked("Show advanced") [
                slider "Detail level" 1..10 @5
            ]
            buttons ["OK"]
        ]"#;
        
        let result = parse_popup_dsl(input);
        assert!(result.is_ok());
        
        let popup = result.unwrap();
        assert_eq!(popup.elements.len(), 3);
        
        match &popup.elements[1] {
            Element::Conditional { condition, elements } => {
                match condition {
                    Condition::Checked(name) => assert_eq!(name, "Show advanced"),
                    _ => panic!("Expected Checked condition"),
                }
                assert_eq!(elements.len(), 1);
            },
            _ => panic!("Expected conditional"),
        }
    }

    #[test]
    fn test_force_yield_auto_added() {
        let input = r#"popup "Test" [
            text "Hello"
            buttons ["OK"]
        ]"#;
        
        let result = parse_popup_dsl(input);
        assert!(result.is_ok());
        
        let popup = result.unwrap();
        match &popup.elements[1] {
            Element::Buttons(buttons) => {
                assert!(buttons.contains(&"Force Yield".to_string()));
            },
            _ => panic!("Expected buttons"),
        }
    }

    #[test]
    fn test_missing_buttons_auto_added() {
        let input = r#"popup "No Buttons" [
            text "This popup has no buttons"
        ]"#;
        
        let result = parse_popup_dsl(input);
        assert!(result.is_ok());
        
        let popup = result.unwrap();
        assert_eq!(popup.elements.len(), 2);
        
        match &popup.elements[1] {
            Element::Buttons(buttons) => {
                assert!(buttons.contains(&"Continue".to_string()));
                assert!(buttons.contains(&"Force Yield".to_string()));
            },
            _ => panic!("Expected buttons"),
        }
    }

    #[test]
    fn test_spike_grounding_check_example() {
        let input = r#"[SPIKE: Quick Grounding Check
            Thing 1: [textbox]
            Thing 2: [textbox]
            Thing 3: [textbox]
            
            One physical sensation you notice: [textbox]
            
            Ready for gentle distraction after?: [Y/N]
        ]"#;
        
        let result = parse_popup_dsl(input);
        if let Err(e) = &result {
            eprintln!("Parse error in test_spike_grounding_check_example: {}", e);
        }
        assert!(result.is_ok());
        
        let popup = result.unwrap();
        assert_eq!(popup.title, "SPIKE: Quick Grounding Check");
        
        // Should have 5 elements + auto-added buttons
        assert!(popup.elements.len() >= 5);
    }

    #[test]
    fn test_complex_conditions() {
        let input = r#"popup "Complex Conditions" [
            choice "Mode" ["Basic", "Advanced"]
            multiselect "Features" ["A", "B", "C"]
            
            if selected("Mode", "Advanced") [
                slider "Power" 1..100 @50
            ]
            
            if count("Features") > 1 [
                text "Multiple features selected!"
            ]
            
            buttons ["Apply"]
        ]"#;
        
        let result = parse_popup_dsl(input);
        assert!(result.is_ok());
        
        let popup = result.unwrap();
        assert_eq!(popup.elements.len(), 5);
    }

    #[test]
    fn test_parse_errors() {
        // Missing closing bracket
        let input = r#"popup "Bad" ["#;
        assert!(parse_popup_dsl(input).is_err());
        
        // Invalid widget type
        let input = r#"[Title: invalid_widget "test"]"#;
        assert!(parse_popup_dsl(input).is_err());
        
        // Missing quotes in classic syntax
        let input = r#"popup Title []"#;
        assert!(parse_popup_dsl(input).is_err());
    }

    #[test]
    fn test_simple_text_minimal() {
        let input = r#"[Test:
            Name: [textbox]
        ]"#;
        
        let result = parse_popup_dsl(input);
        if let Err(e) = &result {
            eprintln!("Parse error in test_simple_text_minimal: {}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_nested_groups() {
        let input = r#"popup "Nested Groups" [
            group "Outer" [
                text "In outer group"
                group "Inner" [
                    checkbox "Deep option" @false
                ]
            ]
            buttons ["OK"]
        ]"#;
        
        let result = parse_popup_dsl(input);
        assert!(result.is_ok());
        
        let popup = result.unwrap();
        match &popup.elements[0] {
            Element::Group { label, elements } => {
                assert_eq!(label, "Outer");
                assert_eq!(elements.len(), 2);
                
                match &elements[1] {
                    Element::Group { label, elements } => {
                        assert_eq!(label, "Inner");
                        assert_eq!(elements.len(), 1);
                    },
                    _ => panic!("Expected inner group"),
                }
            },
            _ => panic!("Expected outer group"),
        }
    }
}
