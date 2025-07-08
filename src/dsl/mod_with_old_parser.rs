use anyhow::Result;
use pest::Parser;
use pest_derive::Parser;

use crate::models::{Element, PopupDefinition, Condition, ComparisonOp};

// New unified parser module
mod unified_parser;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod parser_tests;

#[cfg(test)]
mod edge_case_tests;

#[cfg(test)]
mod new_grammar_tests;

#[cfg(test)]
mod current_grammar_tests;

#[cfg(test)]
mod debug_tests;

#[cfg(test)]
mod debug_tests2;

#[cfg(test)]
mod ast_verification_tests;

#[cfg(test)]
mod unified_tests;

#[cfg(test)]
mod unified_integration_test;

#[derive(Parser)]
#[grammar = "popup.pest"]
pub struct PopupParser;

// Error recovery helpers
mod recovery {
    use super::*;
    
    pub fn try_parse_with_recovery(input: &str) -> Result<PopupDefinition> {
        // Use unified parser directly - it already has good error handling
        crate::dsl::unified_parser::parse_popup_dsl(input)
    }
    
    fn apply_recovery_transforms(input: &str) -> String {
        let mut result = input.to_string();
        
        // 1. Add missing quotes
        result = add_missing_quotes(result);
        
        // 2. Fix common typos
        result = fix_common_typos(result);
        
        // 3. Infer missing colons
        result = infer_missing_colons(result);
        
        // 4. Add default buttons if missing
        result = ensure_buttons(result);
        
        result
    }
    
    fn add_missing_quotes(input: String) -> String {
        // Simple heuristic: add quotes to unquoted strings after keywords
        let keywords = ["popup", "checkbox", "slider", "choice", "text", "group"];
        let mut result = input;
        
        for keyword in &keywords {
            let pattern = format!(r#"{} ([^"'\[\]{{}}:]+)"#, keyword);
            let re = regex::Regex::new(&pattern).unwrap();
            result = re.replace_all(&result, |caps: &regex::Captures| {
                format!("{} \"{}\"", keyword, caps[1].trim())
            }).to_string();
        }
        
        result
    }
    
    fn fix_common_typos(input: String) -> String {
        let typo_map = [
            ("chekbox", "checkbox"),
            ("checkox", "checkbox"),
            ("choise", "choice"),
            ("chioce", "choice"),
            ("mutliselect", "multiselect"),
            ("mutiselect", "multiselect"),
            ("butons", "buttons"),
            ("buton", "button"),
            ("btns", "buttons"),
            ("slidr", "slider"),
            ("rang", "range"),
            ("textbx", "textbox"),
        ];
        
        let mut result = input;
        for (typo, correct) in &typo_map {
            result = result.replace(typo, correct);
        }
        
        result
    }
    
    fn infer_missing_colons(input: String) -> String {
        // Add colon after title if missing
        let lines: Vec<&str> = input.lines().collect();
        if lines.is_empty() {
            return input;
        }
        
        let mut result = Vec::new();
        let first_line = lines[0].trim();
        
        // Check if first line looks like a title without colon
        if !first_line.contains(':') && !first_line.starts_with('[') {
            result.push(format!("{}:", first_line));
        } else {
            result.push(first_line.to_string());
        }
        
        // Add remaining lines
        for line in &lines[1..] {
            result.push(line.to_string());
        }
        
        result.join("\n")
    }
    
    fn ensure_buttons(input: String) -> String {
        // Check if input contains any button patterns
        let button_patterns = ["[", "→", "buttons:", "actions:", "or"];
        let has_buttons = button_patterns.iter().any(|p| input.contains(p));
        
        if !has_buttons {
            // Add default OK button
            format!("{}\n  [OK]", input.trim())
        } else {
            input
        }
    }
}

// Main parsing function with error recovery
pub fn parse_popup_dsl(input: &str) -> Result<PopupDefinition> {
    // Use the new unified parser
    unified_parser::parse_popup_dsl(input)
        .map(|mut def| {
            ensure_button_safety(&mut def);
            def
        })
}

// Internal parsing without recovery - now delegated to unified_parser
fn parse_popup_dsl_internal(input: &str) -> Result<PopupDefinition> {
    unified_parser::parse_popup_dsl(input)
}

fn parse_popup(pair: pest::iterators::Pair<Rule>) -> Result<PopupDefinition> {
    let inner_pair = pair.into_inner().next().unwrap();
    
    match inner_pair.as_rule() {
        Rule::popup_content => parse_popup_content(inner_pair),
        _ => Err(anyhow::anyhow!("Invalid popup structure")),
    }
}

fn parse_popup_content(pair: pest::iterators::Pair<Rule>) -> Result<PopupDefinition> {
    let inner_pair = pair.into_inner().next().unwrap();
    
    match inner_pair.as_rule() {
        Rule::structured_popup => parse_structured_popup(inner_pair),
        Rule::bracket_popup => parse_bracket_popup(inner_pair),
        Rule::natural_popup => parse_natural_popup(inner_pair),
        _ => Err(anyhow::anyhow!("Unknown popup format")),
    }
}

fn parse_structured_popup(pair: pest::iterators::Pair<Rule>) -> Result<PopupDefinition> {
    let mut inner = pair.into_inner();
    
    let title = parse_text_value(inner.next().unwrap())?;
    let body_pair = inner.next().unwrap();
    let elements = parse_body(body_pair)?;
    
    Ok(PopupDefinition { title, elements })
}

fn parse_bracket_popup(pair: pest::iterators::Pair<Rule>) -> Result<PopupDefinition> {
    let mut inner = pair.into_inner();
    
    let title = parse_text_value(inner.next().unwrap())?;
    let body_pair = inner.next().unwrap();
    let elements = parse_inline_body(body_pair)?;
    
    Ok(PopupDefinition { title, elements })
}

fn parse_natural_popup(pair: pest::iterators::Pair<Rule>) -> Result<PopupDefinition> {
    let mut inner = pair.into_inner();
    
    // Skip "confirm"
    let title = parse_text_value(inner.next().unwrap())?;
    let natural_button_list = inner.next().unwrap();
    let buttons = parse_natural_button_list(natural_button_list)?;
    
    Ok(PopupDefinition {
        title,
        elements: vec![Element::Buttons(buttons)],
    })
}

fn parse_body(pair: pest::iterators::Pair<Rule>) -> Result<Vec<Element>> {
    let mut elements = Vec::new();
    
    for element_pair in pair.into_inner() {
        if let Some(element) = parse_element(element_pair)? {
            elements.push(element);
        }
    }
    
    Ok(elements)
}

fn parse_inline_body(pair: pest::iterators::Pair<Rule>) -> Result<Vec<Element>> {
    parse_body(pair)
}

fn parse_element(pair: pest::iterators::Pair<Rule>) -> Result<Option<Element>> {
    match pair.as_rule() {
        Rule::widget => parse_widget(pair).map(Some),
        Rule::buttons => parse_buttons(pair).map(Some),
        Rule::conditional => parse_conditional(pair).map(Some),
        Rule::section => parse_section(pair),
        _ => Ok(None),
    }
}

fn parse_widget(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let inner_pair = pair.into_inner().next().unwrap();
    
    match inner_pair.as_rule() {
        Rule::explicit_widget => parse_explicit_widget(inner_pair),
        Rule::inferred_widget => parse_inferred_widget(inner_pair),
        Rule::natural_widget => parse_natural_widget(inner_pair),
        Rule::symbolic_widget => parse_symbolic_widget(inner_pair),
        Rule::standalone_text => parse_standalone_text(inner_pair),
        _ => Err(anyhow::anyhow!("Unknown widget type")),
    }
}

fn parse_explicit_widget(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let mut inner = pair.into_inner();
    
    let widget_type = inner.next().unwrap();
    let widget_def = inner.next().unwrap();
    
    let widget_kind = normalize_widget_type(widget_type.as_str());
    parse_widget_by_type(&widget_kind, widget_def)
}

fn normalize_widget_type(alias: &str) -> String {
    match alias {
        "checkbox" | "check" | "tick" | "toggle" | "switch" | 
        "bool" | "boolean" | "yes/no" | "y/n" | "enabled" => "checkbox",
        
        "slider" | "range" | "scale" | "numeric" | "number" |
        "dial" | "knob" | "level" | "gauge" | "meter" => "slider",
        
        "textbox" | "input" | "field" | "entry" |
        "textarea" | "string" | "prompt" | "write" => "textbox",
        
        "choice" | "select" | "dropdown" | "pick" | "choose" |
        "option" | "radio" | "single" => "choice",
        
        "multiselect" | "multi" | "multiple" | "checklist" |
        "pickMany" | "selectMultiple" | "options" | "tags" => "multiselect",
        
        "text" | "label" | "message" | "info" | "display" => "text",
        
        "group" | "section" | "container" | "panel" | "box" => "group",
        
        _ => alias,
    }.to_string()
}

fn parse_widget_by_type(widget_type: &str, pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let mut inner = pair.into_inner();
    let label = parse_text_value(inner.next().unwrap())?;
    
    match widget_type {
        "checkbox" => {
            let default = if let Some(options) = inner.next() {
                parse_boolean_default(options)?.unwrap_or(false)
            } else {
                false
            };
            Ok(Element::Checkbox { label, default })
        }
        "slider" => {
            let (min, max, default) = if let Some(options) = inner.next() {
                let (min, max, opt_default) = parse_range_options(options)?;
                (min as f32, max as f32, opt_default.unwrap_or((min + max) / 2.0) as f32)
            } else {
                (0.0, 100.0, 50.0)
            };
            Ok(Element::Slider { label, min, max, default })
        }
        "textbox" => {
            let placeholder = if let Some(options) = inner.next() {
                if options.as_str().starts_with('@') {
                    Some(options.as_str()[1..].trim().to_string())
                } else {
                    Some(parse_text_value(options)?)
                }
            } else {
                None
            };
            Ok(Element::Textbox { label, placeholder, rows: None })
        }
        "choice" => {
            let options = if let Some(list) = inner.next() {
                parse_choice_options(list)?
            } else {
                vec![]
            };
            Ok(Element::Choice { label, options })
        }
        "multiselect" => {
            let options = if let Some(list) = inner.next() {
                parse_choice_options(list)?
            } else {
                vec![]
            };
            Ok(Element::Multiselect { label, options })
        }
        "text" => {
            Ok(Element::Text(label))
        }
        "group" => {
            // Groups need special handling - they contain other elements
            Ok(Element::Group { label, elements: vec![] })
        }
        _ => Err(anyhow::anyhow!("Unknown widget type: {}", widget_type)),
    }
}

fn parse_inferred_widget(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let mut inner = pair.into_inner();
    
    let label = inner.next().unwrap().as_str().trim().to_string();
    let value_pair = inner.next().unwrap();
    
    parse_widget_from_value(label, value_pair)
}

fn parse_widget_from_value(label: String, pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let inner_pair = pair.into_inner().next().unwrap();
    
    match inner_pair.as_rule() {
        Rule::range_pattern => {
            let (min, max, opt_default) = parse_range_pattern(inner_pair)?;
            let default = opt_default.unwrap_or((min + max) / 2.0);
            Ok(Element::Slider { 
                label, 
                min: min as f32, 
                max: max as f32, 
                default: default as f32 
            })
        }
        Rule::boolean_pattern => {
            let default = parse_boolean_pattern(inner_pair)?.unwrap_or(false);
            Ok(Element::Checkbox { label, default })
        }
        Rule::choice_list => {
            let inner = inner_pair.into_inner().next().unwrap();
            match inner.as_rule() {
                Rule::single_choice => {
                    let options = parse_single_choice(inner)?;
                    Ok(Element::Choice { label, options })
                }
                Rule::multi_choice => {
                    let options = parse_multi_choice(inner)?;
                    Ok(Element::Multiselect { label, options })
                }
                _ => Err(anyhow::anyhow!("Unknown choice type")),
            }
        }
        _ => {
            // Check for @ prefix for textbox
            let text = parse_text_value(inner_pair)?;
            if text.starts_with('@') {
                Ok(Element::Textbox {
                    label,
                    placeholder: Some(text[1..].to_string()),
                    rows: None,
                })
            } else {
                Ok(Element::Text(format!("{}: {}", label, text)))
            }
        }
    }
}

fn parse_natural_widget(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let inner_pair = pair.into_inner().next().unwrap();
    
    match inner_pair.as_rule() {
        Rule::natural_slider => parse_natural_slider(inner_pair),
        Rule::natural_checkbox => parse_natural_checkbox(inner_pair),
        Rule::natural_choice => parse_natural_choice(inner_pair),
        _ => Err(anyhow::anyhow!("Unknown natural widget type")),
    }
}

fn parse_natural_slider(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let mut inner = pair.into_inner();
    
    let label = inner.next().unwrap().as_str().trim().to_string();
    let min: f64 = inner.next().unwrap().as_str().parse()?;
    let max: f64 = inner.next().unwrap().as_str().parse()?;
    
    let default = if let Some(default_pair) = inner.next() {
        parse_slider_default(default_pair)?
    } else {
        (min + max) / 2.0
    };
    
    Ok(Element::Slider { 
        label, 
        min: min as f32, 
        max: max as f32, 
        default: default as f32 
    })
}

fn parse_natural_checkbox(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let mut inner = pair.into_inner();
    let first = inner.next().unwrap();
    
    let (label, default) = if first.as_rule() == Rule::checkbox_symbol {
        let symbol = first.as_str();
        let label = inner.next().unwrap().as_str().trim().to_string();
        let default = parse_checkbox_symbol_default(symbol).unwrap_or(false);
        (label, default)
    } else {
        let label = first.as_str().trim().to_string();
        let symbol = inner.next().unwrap().as_str();
        let default = parse_checkbox_symbol_default(symbol).unwrap_or(false);
        (label, default)
    };
    
    Ok(Element::Checkbox { label, default })
}

fn parse_natural_choice(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let mut inner = pair.into_inner();
    
    let label = inner.next().unwrap().as_str().trim().to_string();
    let choice_pattern = inner.next().unwrap();
    let options = parse_single_choice(choice_pattern)?;
    
    Ok(Element::Choice { label, options })
}

fn parse_symbolic_widget(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let mut inner = pair.into_inner();
    let inner_pair = inner.next().unwrap();
    
    match inner_pair.as_rule() {
        Rule::checkbox_symbol => {
            let symbol = inner_pair.as_str();
            let label = inner.next().unwrap().as_str().trim().to_string();
            let default = parse_checkbox_symbol_default(symbol).unwrap_or(false);
            Ok(Element::Checkbox { label, default })
        }
        Rule::star_rating => {
            let stars = inner_pair.as_str();
            let filled = stars.chars().filter(|&c| c == '★').count();
            let total = stars.len();
            Ok(Element::Slider {
                label: "Rating".to_string(),
                min: 0.0,
                max: total as f32,
                default: filled as f32,
            })
        }
        Rule::progress_bar => {
            let bar = inner_pair.as_str();
            let filled = bar.chars().filter(|&c| c == '•' || c == '●' || c == '=').count();
            let total = bar.len() - 2; // Subtract brackets
            Ok(Element::Slider {
                label: "Progress".to_string(),
                min: 0.0,
                max: total as f32,
                default: filled as f32,
            })
        }
        _ => Err(anyhow::anyhow!("Unknown symbolic widget")),
    }
}

fn parse_standalone_text(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let text = pair.as_str();
    
    let formatted = if text.starts_with('>') {
        format!("ℹ️ {}", text[1..].trim())
    } else if text.starts_with('!') {
        format!("⚠️ {}", text[1..].trim())
    } else if text.starts_with('?') {
        format!("❓ {}", text[1..].trim())
    } else if text.starts_with('•') {
        format!("• {}", text[1..].trim())
    } else {
        text.trim().to_string()
    };
    
    Ok(Element::Text(formatted))
}

fn parse_buttons(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let inner_pair = pair.into_inner().next().unwrap();
    
    let labels = match inner_pair.as_rule() {
        Rule::explicit_buttons => {
            let mut inner = inner_pair.into_inner();
            // Skip button alias
            inner.next();
            let button_list = inner.next().unwrap();
            parse_button_list(button_list)?
        }
        Rule::arrow_button => {
            let mut inner = inner_pair.into_inner();
            let button = parse_button_value(inner.next().unwrap())?;
            vec![button]
        }
        Rule::button_row => {
            parse_button_row(inner_pair)?
        }
        Rule::separator_buttons => {
            let inner = inner_pair.into_inner();
            let mut buttons = vec![];
            for item in inner {
                if item.as_rule() == Rule::button_value {
                    buttons.push(parse_button_value(item)?);
                }
            }
            buttons
        }
        Rule::natural_buttons => {
            let mut inner = inner_pair.into_inner();
            vec![
                parse_button_value(inner.next().unwrap())?,
                parse_button_value(inner.next().unwrap())?,
            ]
        }
        _ => vec![],
    };
    
    Ok(Element::Buttons(labels))
}

fn parse_button_list(pair: pest::iterators::Pair<Rule>) -> Result<Vec<String>> {
    let mut buttons = vec![];
    
    for item in pair.into_inner() {
        if item.as_rule() == Rule::button_value {
            buttons.push(parse_button_value(item)?);
        }
    }
    
    Ok(buttons)
}

fn parse_natural_button_list(pair: pest::iterators::Pair<Rule>) -> Result<Vec<String>> {
    let mut buttons = vec![];
    
    for item in pair.into_inner() {
        if item.as_rule() == Rule::natural_button_value {
            buttons.push(parse_natural_button_value(item)?);
        }
    }
    
    Ok(buttons)
}

fn parse_natural_button_value(pair: pest::iterators::Pair<Rule>) -> Result<String> {
    let inner = pair.into_inner().next().unwrap();
    
    match inner.as_rule() {
        Rule::quoted_string => parse_quoted_string(inner),
        Rule::natural_button_text => Ok(inner.as_str().trim().to_string()),
        _ => Ok(inner.as_str().to_string()),
    }
}

fn parse_button_row(pair: pest::iterators::Pair<Rule>) -> Result<Vec<String>> {
    parse_button_list(pair)
}

fn parse_button_value(pair: pest::iterators::Pair<Rule>) -> Result<String> {
    let inner = pair.into_inner().next().unwrap();
    
    match inner.as_rule() {
        Rule::quoted_string => parse_quoted_string(inner),
        Rule::button_text => Ok(inner.as_str().trim().to_string()),
        _ => Ok(inner.as_str().to_string()),
    }
}

fn parse_conditional(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let mut inner = pair.into_inner();
    
    // Skip conditional keyword
    inner.next();
    
    let condition = parse_condition(inner.next().unwrap())?;
    let body = parse_conditional_body(inner.next().unwrap())?;
    
    Ok(Element::Conditional {
        condition,
        elements: body,
    })
}

fn parse_condition(pair: pest::iterators::Pair<Rule>) -> Result<Condition> {
    let inner_pair = pair.into_inner().next().unwrap();
    
    match inner_pair.as_rule() {
        Rule::compound_condition => parse_compound_condition(inner_pair),
        Rule::comparison => parse_comparison(inner_pair),
        Rule::simple_condition => parse_simple_condition(inner_pair),
        Rule::negated_condition => parse_negated_condition(inner_pair),
        _ => Err(anyhow::anyhow!("Unknown condition type")),
    }
}

fn parse_compound_condition(pair: pest::iterators::Pair<Rule>) -> Result<Condition> {
    // For now, just parse the first condition
    // TODO: Add proper compound condition support
    let mut inner = pair.into_inner();
    parse_condition(inner.next().unwrap())
}

fn parse_comparison(pair: pest::iterators::Pair<Rule>) -> Result<Condition> {
    let mut inner = pair.into_inner();
    
    let value_ref = inner.next().unwrap();
    let field = parse_value_reference(value_ref)?;
    
    let op = parse_comparison_op(inner.next().unwrap())?;
    let value = parse_comparison_value(inner.next().unwrap())?;
    
    Ok(Condition::Count(field, op, value as i32))
}

fn parse_simple_condition(pair: pest::iterators::Pair<Rule>) -> Result<Condition> {
    let mut inner = pair.into_inner();
    let value_ref = inner.next().unwrap();
    let field = parse_value_reference(value_ref)?;
    
    Ok(Condition::Checked(field))
}

fn parse_negated_condition(pair: pest::iterators::Pair<Rule>) -> Result<Condition> {
    let mut inner = pair.into_inner();
    // Skip "not" or "!"
    inner.next();
    // For now, just return the inner condition
    // TODO: Add proper negation support
    parse_condition(inner.next().unwrap())
}

fn parse_value_reference(pair: pest::iterators::Pair<Rule>) -> Result<String> {
    let text = pair.as_str();
    
    if text.ends_with(".count") {
        Ok(text[..text.len() - 6].to_string())
    } else if text.starts_with('#') {
        Ok(text[1..].to_string())
    } else if text.starts_with('$') || text.starts_with('@') {
        Ok(text[1..].to_string())
    } else if text.starts_with('{') && text.ends_with('}') {
        Ok(text[1..text.len() - 1].to_string())
    } else {
        Ok(text.to_string())
    }
}

fn parse_comparison_op(pair: pest::iterators::Pair<Rule>) -> Result<ComparisonOp> {
    match pair.as_str() {
        ">" | "more than" => Ok(ComparisonOp::Greater),
        "<" | "less than" => Ok(ComparisonOp::Less),
        ">=" | "at least" => Ok(ComparisonOp::GreaterEqual),
        "<=" => Ok(ComparisonOp::LessEqual),
        "=" | "==" => Ok(ComparisonOp::Equal),
        "!=" => Err(anyhow::anyhow!("NotEqual operator not supported in current model")),
        _ => Err(anyhow::anyhow!("Unknown comparison operator")),
    }
}

fn parse_comparison_value(pair: pest::iterators::Pair<Rule>) -> Result<f64> {
    let inner = pair.into_inner().next().unwrap();
    
    match inner.as_rule() {
        Rule::number => inner.as_str().parse().map_err(Into::into),
        _ => Ok(0.0), // Default for non-numeric comparisons
    }
}

fn parse_conditional_body(pair: pest::iterators::Pair<Rule>) -> Result<Vec<Element>> {
    match pair.as_rule() {
        Rule::indented_body => parse_body(pair),
        _ => {
            // Bracketed body
            let inner = pair.into_inner().next().unwrap();
            parse_body(inner)
        }
    }
}

fn parse_section(pair: pest::iterators::Pair<Rule>) -> Result<Option<Element>> {
    let mut inner = pair.into_inner();
    
    let header = inner.next().unwrap();
    let label = parse_section_header(header)?;
    
    let body = inner.next().unwrap();
    let elements = parse_body(body)?;
    
    Ok(Some(Element::Group { label, elements }))
}

fn parse_section_header(pair: pest::iterators::Pair<Rule>) -> Result<String> {
    let text = pair.as_str();
    
    if text.starts_with("---") && text.ends_with("---") {
        Ok(text.trim_matches('-').trim().to_string())
    } else if text.ends_with(':') {
        Ok(text.trim_end_matches(':').trim().to_string())
    } else {
        Ok(text.trim().to_string())
    }
}

// Helper parsing functions
fn parse_range_pattern(pair: pest::iterators::Pair<Rule>) -> Result<(f64, f64, Option<f64>)> {
    let mut inner = pair.into_inner();
    
    let min: f64 = inner.next().unwrap().as_str().parse()?;
    let max: f64 = inner.next().unwrap().as_str().parse()?;
    
    let default = if let Some(default_pair) = inner.next() {
        Some(parse_range_default(default_pair)?)
    } else {
        None
    };
    
    Ok((min, max, default))
}

fn parse_range_options(pair: pest::iterators::Pair<Rule>) -> Result<(f64, f64, Option<f64>)> {
    if pair.as_rule() == Rule::range_pattern {
        parse_range_pattern(pair)
    } else {
        Ok((0.0, 100.0, None))
    }
}

fn parse_range_default(pair: pest::iterators::Pair<Rule>) -> Result<f64> {
    let mut inner = pair.into_inner();
    inner.next().unwrap().as_str().parse().map_err(Into::into)
}

fn parse_slider_default(pair: pest::iterators::Pair<Rule>) -> Result<f64> {
    let mut inner = pair.into_inner();
    inner.next().unwrap().as_str().parse().map_err(Into::into)
}

fn parse_boolean_pattern(pair: pest::iterators::Pair<Rule>) -> Result<Option<bool>> {
    let text = pair.as_str();
    
    Ok(match text {
        "yes" | "on" | "true" | "enabled" | "active" => Some(true),
        "no" | "off" | "false" | "disabled" | "inactive" => Some(false),
        s if s.starts_with('✓') || s.starts_with('☑') => Some(true),
        s if s.starts_with('✗') || s.starts_with('☐') || s.starts_with('☒') => Some(false),
        "[x]" | "[X]" | "(*)" => Some(true),
        "[ ]" | "( )" => Some(false),
        _ => None,
    })
}

fn parse_boolean_default(pair: pest::iterators::Pair<Rule>) -> Result<Option<bool>> {
    parse_boolean_pattern(pair)
}

fn parse_checkbox_symbol_default(symbol: &str) -> Option<bool> {
    match symbol {
        "✓" | "☑" | "[x]" | "[X]" | "(*)" => Some(true),
        "✗" | "☐" | "☒" | "[ ]" | "( )" => Some(false),
        _ => None,
    }
}

fn parse_choice_options(pair: pest::iterators::Pair<Rule>) -> Result<Vec<String>> {
    if pair.as_rule() == Rule::choice_list {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::single_choice => parse_single_choice(inner),
            Rule::multi_choice => parse_multi_choice(inner),
            _ => Ok(vec![]),
        }
    } else {
        Ok(vec![])
    }
}

fn parse_single_choice(pair: pest::iterators::Pair<Rule>) -> Result<Vec<String>> {
    let mut options = vec![];
    
    for item in pair.into_inner() {
        if item.as_rule() == Rule::choice_value {
            options.push(parse_choice_value(item)?);
        }
    }
    
    Ok(options)
}

fn parse_multi_choice(pair: pest::iterators::Pair<Rule>) -> Result<Vec<String>> {
    parse_single_choice(pair)
}

fn parse_choice_value(pair: pest::iterators::Pair<Rule>) -> Result<String> {
    let inner = pair.into_inner().next().unwrap();
    
    match inner.as_rule() {
        Rule::quoted_string => parse_quoted_string(inner),
        Rule::word => Ok(inner.as_str().to_string()),
        _ => Ok(inner.as_str().to_string()),
    }
}

fn parse_text_value(pair: pest::iterators::Pair<Rule>) -> Result<String> {
    let inner = pair.into_inner().next().unwrap();
    
    match inner.as_rule() {
        Rule::quoted_string => parse_quoted_string(inner),
        Rule::unquoted_text => Ok(inner.as_str().trim().to_string()),
        _ => Ok(inner.as_str().to_string()),
    }
}

fn parse_quoted_string(pair: pest::iterators::Pair<Rule>) -> Result<String> {
    let mut inner = pair.into_inner();
    let content = inner.next().unwrap();
    Ok(content.as_str().to_string())
}

fn ensure_button_safety(definition: &mut PopupDefinition) {
    let mut has_buttons = false;
    let mut has_force_yield = false;
    
    for element in &definition.elements {
        if let Element::Buttons(labels) = element {
            has_buttons = true;
            if labels.iter().any(|l| l == "Force Yield") {
                has_force_yield = true;
            }
        }
    }
    
    if !has_buttons {
        eprintln!("Warning: No buttons found. Adding default [OK] button.");
        definition.elements.push(Element::Buttons(vec!["OK".to_string()]));
        has_buttons = true;
    }
    
    if has_buttons && !has_force_yield {
        for element in &mut definition.elements {
            if let Element::Buttons(ref mut labels) = element {
                labels.push("Force Yield".to_string());
                break;
            }
        }
    }
}

fn format_helpful_error(_input: &str, error: &pest::error::Error<Rule>) -> String {
    let base = error.to_string();
    
    let help = "\n\nPopup DSL supports multiple formats:\n\n\
        1. Structured format:\n\
           Title:\n\
             Volume: 0-100\n\
             Theme: Light | Dark\n\
             [Save | Cancel]\n\n\
        2. Natural language:\n\
           confirm \"Delete file?\" with Yes or No\n\n\
        3. Inline format:\n\
           [Settings: Volume: 0-100, Save or Cancel]\n\n\
        Common widget patterns:\n\
        - Slider: 'Volume: 0-100' or 'range Volume from 0 to 100'\n\
        - Checkbox: 'Enabled: yes' or '✓ Notifications'\n\
        - Choice: 'Theme: Light | Dark'\n\
        - Textbox: 'Name: @Enter your name'\n\
        - Buttons: '[OK | Cancel]' or '→ Continue'";
    
    format!("{}\n{}", base, help)
}

// Serialization remains mostly the same
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