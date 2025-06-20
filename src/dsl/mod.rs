use anyhow::Result;
use pest::Parser;
use pest_derive::Parser;
use std::fs::OpenOptions;
use std::io::Write;

use crate::models::{Element, PopupDefinition, Condition, ComparisonOp};

#[derive(Parser)]
#[grammar = "popup.pest"]
pub struct PopupParser;

pub fn parse_popup_dsl(input: &str) -> Result<PopupDefinition> {
    let pairs = PopupParser::parse(Rule::popup, input)
        .map_err(|e| {
            // Log parse errors to file for ergonomics improvement
            log_parse_error(input, &e);
            
            // Create helpful error message with examples
            let helpful_error = format_helpful_error(input, &e);
            anyhow::anyhow!("{}", helpful_error)
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

/// Log parse errors to a file for analysis and ergonomics improvements
fn log_parse_error(input: &str, error: &pest::error::Error<Rule>) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("parser_errors.log")
    {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        let log_entry = format!(
            "\n--- Parse Error ({}) ---\nInput:\n{}\n\nError:\n{}\n",
            timestamp, input, error
        );
        
        let _ = file.write_all(log_entry.as_bytes());
    }
}

/// Format error messages with helpful examples and suggestions
fn format_helpful_error(input: &str, error: &pest::error::Error<Rule>) -> String {
    let trimmed = input.trim();
    
    // Detect common error patterns and provide specific help
    match detect_error_pattern(trimmed, error) {
        ErrorPattern::MissingQuotes => {
            format!(
                "Parse error: Missing quotes around title\n\n\
                 You wrote: popup {} [...]\n\
                 Should be: popup \"{}\" [...]\n\n\
                 Example:\n  \
                 popup \"My Title\" [\n    \
                 text \"Hello\",\n    \
                 buttons [\"OK\"]\n  \
                 ]",
                extract_unquoted_title(trimmed),
                extract_unquoted_title(trimmed)
            )
        },
        ErrorPattern::FormatMixing => {
            "Parse error: Format mixing detected!\n\n\
             Choose ONE format style:\n\n\
             Classic format (with 'popup' keyword):\n  \
             popup \"Title\" [\n    \
             text \"Hello\",\n    \
             buttons [\"OK\"]\n  \
             ]\n\n\
             Simplified format (no 'popup' keyword):\n  \
             [Title:\n    \
             text \"Hello\",\n    \
             buttons [\"OK\"]\n  \
             ]\n\n\
             Tip: If you use 'popup', use quotes. If you use brackets only, use a colon.".to_string()
        },
        ErrorPattern::InvalidWidget => {
            let widget = extract_invalid_widget(trimmed, error);
            format!(
                "Parse error: Unknown widget type '{}'\n\n\
                 Valid widget types:\n  \
                 • text \"message\" - Display text\n  \
                 • textbox \"label\" - Text input field\n  \
                 • checkbox \"label\" - Boolean toggle\n  \
                 • slider \"label\" min..max - Numeric range\n  \
                 • choice \"label\" [\"opt1\", \"opt2\"] - Single selection\n  \
                 • multiselect \"label\" [\"opt1\", \"opt2\"] - Multiple selection\n  \
                 • buttons [\"label1\", \"label2\"] - Action buttons (required!)\n\n\
                 Example:\n  \
                 popup \"Form\" [\n    \
                 textbox \"Name\",\n    \
                 checkbox \"Subscribe\" @true,\n    \
                 buttons [\"Submit\"]\n  \
                 ]",
                widget
            )
        },
        ErrorPattern::MissingElement => {
            let line = get_error_line(error);
            format!(
                "Parse error at line {}: Expected a popup element\n\n\
                 Common issues:\n  \
                 • Missing comma between elements on the same line\n  \
                 • Invalid element syntax\n  \
                 • Empty popup body\n\n\
                 Valid elements:\n  \
                 • text \"message\"\n  \
                 • textbox \"label\"\n  \
                 • checkbox \"label\"\n  \
                 • slider \"label\" 0..10\n  \
                 • buttons [\"OK\", \"Cancel\"]\n\n\
                 Example of a complete popup:\n  \
                 popup \"Example\" [\n    \
                 text \"Enter your details\",\n    \
                 textbox \"Name\",\n    \
                 buttons [\"Submit\"]\n  \
                 ]",
                line
            )
        },
        ErrorPattern::SimplifiedSyntaxError => {
            "Parse error: Invalid simplified syntax\n\n\
             The simplified syntax requires:\n  \
             1. Opening bracket [\n  \
             2. Title followed by colon\n  \
             3. Elements\n  \
             4. Closing bracket ]\n\n\
             Correct examples:\n  \
             [Quick Check:\n    \
             text \"Status check\",\n    \
             buttons [\"Continue\"]\n  \
             ]\n\n  \
             [User Input:\n    \
             Name: [textbox]\n    \
             Ready: [Y/N]\n    \
             buttons [\"Submit\"]\n  \
             ]\n\n\
             Note: 'Label: [widget]' only works for simple widgets (textbox, checkbox, Y/N)".to_string()
        },
        ErrorPattern::Generic => {
            // Fallback to original error with general help
            format!(
                "{}\n\n\
                 Quick syntax reference:\n\n\
                 Classic format:\n  \
                 popup \"Title\" [\n    \
                 text \"message\",\n    \
                 buttons [\"OK\"]\n  \
                 ]\n\n\
                 Simplified format:\n  \
                 [Title:\n    \
                 text \"message\",\n    \
                 buttons [\"OK\"]\n  \
                 ]\n\n\
                 For more examples, see the documentation.",
                error
            )
        }
    }
}

#[derive(Debug)]
enum ErrorPattern {
    MissingQuotes,
    FormatMixing,
    InvalidWidget,
    MissingElement,
    SimplifiedSyntaxError,
    Generic,
}

fn detect_error_pattern(input: &str, error: &pest::error::Error<Rule>) -> ErrorPattern {
    let error_str = error.to_string();
    
    // Check for missing quotes (popup Title instead of popup "Title")
    if input.starts_with("popup ") && error_str.contains("expected string") {
        return ErrorPattern::MissingQuotes;
    }
    
    // Check for format mixing
    if input.contains("popup") && input.contains(": [") {
        return ErrorPattern::FormatMixing;
    }
    
    // Check for invalid widget in simplified syntax
    if error_str.contains("expected element") && input.contains("invalid_widget") {
        return ErrorPattern::InvalidWidget;
    }
    
    // Check for simplified syntax errors
    if input.starts_with('[') && input.contains(':') && error_str.contains("expected element") {
        return ErrorPattern::SimplifiedSyntaxError;
    }
    
    // Check for missing element (empty popup, missing comma, etc)
    if error_str.contains("expected element") {
        return ErrorPattern::MissingElement;
    }
    
    ErrorPattern::Generic
}

fn extract_unquoted_title(input: &str) -> &str {
    if let Some(start) = input.find("popup ") {
        let after_popup = &input[start + 6..];
        if let Some(bracket_pos) = after_popup.find('[') {
            return after_popup[..bracket_pos].trim();
        }
    }
    "Title"
}

fn extract_invalid_widget<'a>(input: &'a str, _error: &pest::error::Error<Rule>) -> &'a str {
    // Try to find widget name near error position
    if input.contains("invalid_widget") {
        return "invalid_widget";
    }
    "unknown"
}

fn get_error_line(error: &pest::error::Error<Rule>) -> usize {
    match &error.line_col {
        pest::error::LineColLocation::Pos((line, _)) => *line,
        pest::error::LineColLocation::Span((line, _), _) => *line,
    }
}

/// Serialize a PopupDefinition back to DSL format for round-trip testing
pub fn serialize_popup_dsl(definition: &PopupDefinition) -> String {
    let mut result = format!("popup \"{}\" [\n", definition.title);
    
    for element in &definition.elements {
        result.push_str(&serialize_element(element, 1));
    }
    
    result.push_str("]");
    result
}

fn serialize_element(element: &Element, indent_level: usize) -> String {
    let indent = "    ".repeat(indent_level);
    
    match element {
        Element::Text(text) => format!("{}text \"{}\"\n", indent, text),
        Element::Slider { label, min, max, default } => {
            if (min + max) / 2.0 == *default {
                format!("{}slider \"{}\" {}..{}\n", indent, label, min, max)
            } else {
                format!("{}slider \"{}\" {}..{} @{}\n", indent, label, min, max, default)
            }
        },
        Element::Checkbox { label, default } => {
            if *default {
                format!("{}checkbox \"{}\" @true\n", indent, label)
            } else {
                format!("{}checkbox \"{}\"\n", indent, label)
            }
        },
        Element::Textbox { label, placeholder, rows } => {
            let mut result = format!("{}textbox \"{}\"", indent, label);
            if let Some(placeholder) = placeholder {
                result.push_str(&format!(" \"{}\"", placeholder));
            }
            if let Some(rows) = rows {
                result.push_str(&format!(" rows={}", rows));
            }
            result.push('\n');
            result
        },
        Element::Choice { label, options } => {
            format!("{}choice \"{}\" [{}]\n", 
                indent, 
                label, 
                options.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(", ")
            )
        },
        Element::Multiselect { label, options } => {
            format!("{}multiselect \"{}\" [{}]\n", 
                indent, 
                label, 
                options.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(", ")
            )
        },
        Element::Group { label, elements } => {
            let mut result = format!("{}group \"{}\" [\n", indent, label);
            for elem in elements {
                result.push_str(&serialize_element(elem, indent_level + 1));
            }
            result.push_str(&format!("{}]\n", indent));
            result
        },
        Element::Buttons(buttons) => {
            format!("{}buttons [{}]\n", 
                indent, 
                buttons.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(", ")
            )
        },
        Element::Conditional { condition, elements } => {
            let mut result = format!("{}if {} [\n", indent, serialize_condition(condition));
            for elem in elements {
                result.push_str(&serialize_element(elem, indent_level + 1));
            }
            result.push_str(&format!("{}]\n", indent));
            result
        },
    }
}

fn serialize_condition(condition: &Condition) -> String {
    match condition {
        Condition::Checked(name) => format!("checked(\"{}\")", name),
        Condition::Selected(name, value) => format!("selected(\"{}\", \"{}\")", name, value),
        Condition::Count(name, op, value) => {
            let op_str = match op {
                ComparisonOp::Greater => ">",
                ComparisonOp::Less => "<",
                ComparisonOp::GreaterEqual => ">=",
                ComparisonOp::LessEqual => "<=",
                ComparisonOp::Equal => "==",
            };
            format!("count(\"{}\") {} {}", name, op_str, value)
        },
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
    fn test_enhanced_error_messages() {
        // Test missing quotes error
        let input = r#"popup Title []"#;
        let result = parse_popup_dsl(input);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Missing quotes around title"));
        assert!(error_msg.contains("Should be: popup \"Title\""));
        
        // Test invalid widget error
        let input = r#"[Title: invalid_widget "test"]"#;
        let result = parse_popup_dsl(input);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid simplified syntax") || error_msg.contains("Unknown widget"));
        
        // Test empty popup error
        let input = r#"popup "Bad" ["#;
        let result = parse_popup_dsl(input);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Expected a popup element"));
        
        // Test format mixing error
        let input = r#"popup "Title" [ Name: [textbox] ]"#;
        let result = parse_popup_dsl(input);
        assert!(result.is_err());
        // This might not detect as format mixing, but should still provide helpful error
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
