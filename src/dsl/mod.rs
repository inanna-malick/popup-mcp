use anyhow::Result;

use crate::models::{Element, PopupDefinition, Condition, ComparisonOp};

// New unified parser module
mod unified_parser;

// Simple parser with intelligent widget detection
mod simple_parser;

// Temporarily disabled old tests that use old grammar
// #[cfg(test)]
// mod tests;

// #[cfg(test)]
// mod parser_tests;

// #[cfg(test)]
// mod edge_case_tests;

// #[cfg(test)]
// mod new_grammar_tests;

// #[cfg(test)]
// mod current_grammar_tests;

// #[cfg(test)]
// mod debug_tests;

// #[cfg(test)]
// mod debug_tests2;

// #[cfg(test)]
// mod ast_verification_tests;

// #[cfg(test)]
// mod unified_tests;

// #[cfg(test)]
// mod unified_integration_test;

#[cfg(test)]
mod unified_parser_tests;

#[cfg(test)]
mod simple_parser_tests;

// Main parsing function
pub fn parse_popup_dsl(input: &str) -> Result<PopupDefinition> {
    // Use the new simple parser with intelligent widget detection
    simple_parser::parse_popup_dsl(input)
        .map(|mut def| {
            ensure_button_safety(&mut def);
            def
        })
}

// Ensure buttons have Force Yield
fn ensure_button_safety(popup: &mut PopupDefinition) {
    let has_buttons = popup.elements.iter().any(|e| matches!(e, Element::Buttons(_)));
    
    if has_buttons {
        for element in &mut popup.elements {
            if let Element::Buttons(ref mut labels) = element {
                if !labels.contains(&"Force Yield".to_string()) {
                    labels.push("Force Yield".to_string());
                }
                break;
            }
        }
    }
}

// Format helpful error messages for users
fn format_helpful_error(input: &str, error: &pest::error::Error<unified_parser::Rule>) -> String {
    use pest::error::ErrorVariant;
    
    let (line, col) = match error.line_col {
        pest::error::LineColLocation::Pos((l, c)) => (l, c),
        pest::error::LineColLocation::Span((l, c), _) => (l, c),
    };
    
    let lines: Vec<&str> = input.lines().collect();
    let error_line = lines.get(line.saturating_sub(1)).unwrap_or(&"");
    
    let mut message = format!("Parse error at line {}, column {}:\n", line, col);
    message.push_str(&format!("  {}\n", error_line));
    message.push_str(&format!("  {}^\n", " ".repeat(col.saturating_sub(1))));
    
    match &error.variant {
        ErrorVariant::ParsingError { positives, negatives } => {
            if !positives.is_empty() {
                message.push_str("Expected one of: ");
                let expected: Vec<String> = positives.iter()
                    .map(|r| format!("{:?}", r))
                    .collect();
                message.push_str(&expected.join(", "));
            }
        }
        ErrorVariant::CustomError { message: custom } => {
            message.push_str(&format!("Error: {}", custom));
        }
    }
    
    // Add helpful suggestions
    message.push_str("\n\nHint: ");
    if error_line.contains("checkbox") || error_line.contains("check") {
        message.push_str("For checkboxes, use format: 'Label: yes' or 'Label: ✓'");
    } else if error_line.contains("slider") || error_line.contains("..") {
        message.push_str("For sliders, use format: 'Label: 0-100' or 'Label: 0..100 = 50'");
    } else if error_line.contains("|") && !error_line.contains("[") {
        message.push_str("For choices, use format: 'Label: Option1 | Option2'");
    } else if error_line.trim().is_empty() {
        message.push_str("Empty lines are not allowed as elements");
    } else {
        message.push_str("Check the popup syntax - common patterns:\n");
        message.push_str("  - Title:\n");
        message.push_str("  - Widget: value\n");
        message.push_str("  - [Button1 | Button2]");
    }
    
    message
}

// Serialize a popup definition back to DSL format
pub fn serialize_popup_dsl(definition: &PopupDefinition) -> String {
    let mut result = format!("{}:", definition.title);
    
    for element in &definition.elements {
        result.push_str("\n  ");
        result.push_str(&serialize_element(element, 1));
    }
    
    result
}

fn serialize_element(element: &Element, indent: usize) -> String {
    let indent_str = "  ".repeat(indent);
    
    match element {
        Element::Text(text) => {
            if text.starts_with("ℹ️") {
                format!("> {}", text.trim_start_matches("ℹ️ ").trim())
            } else if text.starts_with("⚠️") {
                format!("! {}", text.trim_start_matches("⚠️ ").trim())
            } else if text.starts_with("❓") {
                format!("? {}", text.trim_start_matches("❓ ").trim())
            } else {
                text.clone()
            }
        }
        Element::Slider { label, min, max, default } => {
            format!("{}: {}-{} = {}", label, min, max, default)
        }
        Element::Checkbox { label, default } => {
            if *default {
                format!("{}: ✓", label)
            } else {
                format!("{}: ☐", label)
            }
        }
        Element::Textbox { label, placeholder, rows: _ } => {
            if let Some(ph) = placeholder {
                format!("{}: @{}", label, ph)
            } else {
                format!("{}: @", label)
            }
        }
        Element::Choice { label, options } => {
            format!("{}: {}", label, options.join(" | "))
        }
        Element::Multiselect { label, options } => {
            format!("{}: [{}]", label, options.join(", "))
        }
        Element::Group { label, elements } => {
            let mut result = format!("--- {} ---", label);
            for elem in elements {
                result.push_str(&format!("\n{}{}", indent_str, serialize_element(elem, indent + 1)));
            }
            result
        }
        Element::Buttons(labels) => {
            let filtered: Vec<String> = labels.iter()
                .filter(|l| *l != "Force Yield")
                .cloned()
                .collect();
            
            if filtered.len() == 1 {
                format!("→ {}", filtered[0])
            } else if filtered.is_empty() {
                "[Force Yield]".to_string()
            } else {
                format!("[{}]", filtered.join(" | "))
            }
        }
        Element::Conditional { condition, elements } => {
            let mut result = format!("when {}:", serialize_condition(condition));
            for elem in elements {
                result.push_str(&format!("\n{}{}", indent_str, serialize_element(elem, indent + 1)));
            }
            result
        }
    }
}

fn serialize_condition(condition: &Condition) -> String {
    match condition {
        Condition::Checked(name) => name.clone(),
        Condition::Selected(name, value) => format!("{} = {}", name, value),
        Condition::Count(field, op, value) => {
            let op_str = match op {
                ComparisonOp::Greater => ">",
                ComparisonOp::Less => "<",
                ComparisonOp::GreaterEqual => ">=",
                ComparisonOp::LessEqual => "<=",
                ComparisonOp::Equal => "=",
            };
            format!("#{} {} {}", field, op_str, value)
        }
    }
}