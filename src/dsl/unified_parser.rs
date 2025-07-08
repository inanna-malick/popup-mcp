use anyhow::Result;
use pest::Parser;
use pest_derive::Parser;

use crate::models::{Element, PopupDefinition, Condition, ComparisonOp};

#[derive(Parser)]
#[grammar = "src/popup.pest"]
pub struct PopupParser;

pub fn parse_popup_dsl(input: &str) -> Result<PopupDefinition> {
    let pairs = PopupParser::parse(Rule::popup, input)
        .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;
    
    let popup_pair = pairs.into_iter().next().unwrap();
    parse_popup(popup_pair)
}

fn parse_popup(pair: pest::iterators::Pair<Rule>) -> Result<PopupDefinition> {
    let mut title = String::new();
    let mut elements: Vec<Element> = Vec::new();
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::natural_language_popup => {
                return parse_natural_language_popup(inner);
            }
            Rule::structured_popup => {
                return parse_structured_popup(inner);
            }
            Rule::EOI => {}
            _ => {}
        }
    }
    
    Err(anyhow::anyhow!("Invalid popup format"))
}

fn parse_natural_language_popup(pair: pest::iterators::Pair<Rule>) -> Result<PopupDefinition> {
    let mut title = String::new();
    let mut buttons = Vec::new();
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::title_text => {
                title = parse_title_text(inner)?;
            }
            Rule::natural_buttons => {
                // Parse natural buttons directly
                for button in inner.into_inner() {
                    if button.as_rule() == Rule::button_text {
                        buttons.push(parse_button_text(button)?);
                    }
                }
            }
            _ => {}
        }
    }
    
    let mut elements = vec![Element::Buttons(buttons)];
    ensure_force_yield(&mut elements);
    
    Ok(PopupDefinition { title, elements })
}

fn parse_structured_popup(pair: pest::iterators::Pair<Rule>) -> Result<PopupDefinition> {
    let mut title = String::new();
    let mut elements = Vec::new();
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::title => {
                title = parse_title(inner)?;
            }
            Rule::body => {
                elements = parse_body(inner)?;
            }
            _ => {}
        }
    }
    
    ensure_force_yield(&mut elements);
    
    Ok(PopupDefinition { title, elements })
}

fn parse_title_line(pair: pest::iterators::Pair<Rule>) -> Result<String> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::title {
            return parse_title(inner);
        }
    }
    Err(anyhow::anyhow!("No title found"))
}

fn parse_title(pair: pest::iterators::Pair<Rule>) -> Result<String> {
    // Skip "confirm" if present and get the actual title text
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::title_text {
            return parse_title_text(inner);
        }
    }
    Err(anyhow::anyhow!("No title text found"))
}

fn parse_title_text(pair: pest::iterators::Pair<Rule>) -> Result<String> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::quoted_string => parse_quoted_string(inner),
        Rule::title_unquoted => Ok(inner.as_str().trim().to_string()),
        _ => Ok(inner.as_str().to_string()),
    }
}

fn parse_body(pair: pest::iterators::Pair<Rule>) -> Result<Vec<Element>> {
    let mut elements = Vec::new();
    
    for element_pair in pair.into_inner() {
        if element_pair.as_rule() == Rule::element {
            if let Some(element) = parse_element(element_pair)? {
                elements.push(element);
            }
        }
    }
    
    Ok(elements)
}

fn parse_element(pair: pest::iterators::Pair<Rule>) -> Result<Option<Element>> {
    let inner = pair.into_inner().next().unwrap();
    
    match inner.as_rule() {
        Rule::widget => parse_widget(inner).map(Some),
        Rule::buttons => parse_buttons(inner).map(Some),
        Rule::conditional => parse_conditional(inner).map(Some),
        Rule::section => parse_section(inner),
        Rule::message => parse_message(inner).map(Some),
        _ => Ok(None),
    }
}

fn parse_widget(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let mut inner = pair.into_inner();
    let label = inner.next().unwrap().as_str().trim().to_string();
    let value_pair = inner.next().unwrap();
    
    parse_widget_value(label, value_pair)
}

fn parse_widget_value(label: String, pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let inner = pair.into_inner().next().unwrap();
    
    match inner.as_rule() {
        Rule::range_value => {
            let (min, max, default) = parse_range_value(inner)?;
            Ok(Element::Slider { label, min, max, default })
        }
        Rule::boolean_value => {
            let default = parse_boolean_value(inner)?;
            Ok(Element::Checkbox { label, default })
        }
        Rule::choice_value => {
            let (is_multi, options) = parse_choice_value(inner)?;
            if is_multi {
                Ok(Element::Multiselect { label, options })
            } else {
                Ok(Element::Choice { label, options })
            }
        }
        Rule::textbox_value => {
            let placeholder = parse_textbox_value(inner)?;
            Ok(Element::Textbox { label, placeholder, rows: None })
        }
        Rule::text_value => {
            let text = parse_text_value(inner)?;
            Ok(Element::Text(format!("{}: {}", label, text)))
        }
        _ => Ok(Element::Text(format!("{}: {}", label, inner.as_str())))
    }
}

fn parse_range_value(pair: pest::iterators::Pair<Rule>) -> Result<(f32, f32, f32)> {
    let mut inner = pair.into_inner();
    let min: f32 = inner.next().unwrap().as_str().parse()?;
    
    // Skip separator
    inner.next();
    
    let max: f32 = inner.next().unwrap().as_str().parse()?;
    
    // Check for default value
    let default = if let Some(default_pair) = inner.next() {
        if default_pair.as_rule() == Rule::number {
            default_pair.as_str().parse()?
        } else {
            // Skip "=" or "default" and get the number
            inner.next().unwrap().as_str().parse()?
        }
    } else {
        (min + max) / 2.0
    };
    
    Ok((min, max, default))
}

fn parse_boolean_value(pair: pest::iterators::Pair<Rule>) -> Result<bool> {
    let value = pair.as_str();
    Ok(matches!(value, "yes" | "true" | "on" | "✓" | "☑" | "[x]" | "[X]" | "(*)"))
}

fn parse_choice_value(pair: pest::iterators::Pair<Rule>) -> Result<(bool, Vec<String>)> {
    let is_multi = pair.as_str().starts_with('[');
    let mut options = Vec::new();
    
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::choice_item {
            options.push(parse_choice_item(inner)?);
        }
    }
    
    Ok((is_multi, options))
}

fn parse_choice_item(pair: pest::iterators::Pair<Rule>) -> Result<String> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::quoted_string => parse_quoted_string(inner),
        Rule::choice_text => Ok(inner.as_str().trim().to_string()),
        _ => Ok(inner.as_str().to_string()),
    }
}

fn parse_textbox_value(pair: pest::iterators::Pair<Rule>) -> Result<Option<String>> {
    // Skip the @ symbol and get the optional placeholder
    if let Some(text_pair) = pair.into_inner().next() {
        Ok(Some(parse_text_value(text_pair)?))
    } else {
        Ok(None)
    }
}

fn parse_buttons(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let inner = pair.into_inner().next().unwrap();
    let mut labels = Vec::new();
    
    match inner.as_rule() {
        Rule::natural_buttons => {
            // Skip "with" or "using"
            for button in inner.into_inner() {
                if button.as_rule() == Rule::button_text {
                    labels.push(parse_button_text(button)?);
                }
            }
        }
        Rule::bracket_buttons => {
            for button in inner.into_inner() {
                if button.as_rule() == Rule::button_text {
                    labels.push(parse_button_text(button)?);
                }
            }
        }
        Rule::arrow_button => {
            for button in inner.into_inner() {
                if button.as_rule() == Rule::button_text {
                    labels.push(parse_button_text(button)?);
                }
            }
        }
        Rule::explicit_buttons => {
            // Skip "buttons:" and parse the rest
            for child in inner.into_inner() {
                match child.as_rule() {
                    Rule::bracket_buttons => {
                        for button in child.into_inner() {
                            if button.as_rule() == Rule::button_text {
                                labels.push(parse_button_text(button)?);
                            }
                        }
                    }
                    Rule::button_text => {
                        labels.push(parse_button_text(child)?);
                    }
                    _ => {}
                }
            }
        }
        Rule::separator_buttons => {
            for button in inner.into_inner() {
                if button.as_rule() == Rule::button_text {
                    labels.push(parse_button_text(button)?);
                }
            }
        }
        Rule::standalone_buttons => {
            for button in inner.into_inner() {
                if button.as_rule() == Rule::button_text {
                    labels.push(parse_button_text(button)?);
                }
            }
        }
        _ => {}
    }
    
    Ok(Element::Buttons(labels))
}

fn parse_button_text(pair: pest::iterators::Pair<Rule>) -> Result<String> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::quoted_string => parse_quoted_string(inner),
        Rule::button_unquoted => Ok(inner.as_str().trim().to_string()),
        _ => Ok(inner.as_str().to_string()),
    }
}

fn parse_conditional(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let mut inner = pair.into_inner();
    
    // Skip "when" or "if"
    let condition_pair = inner.find(|p| p.as_rule() == Rule::condition).unwrap();
    let condition = parse_condition(condition_pair)?;
    
    let body_pair = inner.find(|p| p.as_rule() == Rule::conditional_body).unwrap();
    let elements = parse_conditional_body(body_pair)?;
    
    Ok(Element::Conditional { condition, elements })
}

fn parse_condition(pair: pest::iterators::Pair<Rule>) -> Result<Condition> {
    let mut inner = pair.into_inner();
    let field_name = inner.next().unwrap().as_str().trim().to_string();
    
    // Check what follows the field name
    if let Some(next) = inner.next() {
        match next.as_str() {
            "=" => {
                let value = parse_field_value(inner.next().unwrap())?;
                Ok(Condition::Selected(field_name, value))
            }
            ">" => {
                let value: i32 = inner.next().unwrap().as_str().parse()?;
                Ok(Condition::Count(field_name, ComparisonOp::Greater, value))
            }
            "<" => {
                let value: i32 = inner.next().unwrap().as_str().parse()?;
                Ok(Condition::Count(field_name, ComparisonOp::Less, value))
            }
            ">=" => {
                let value: i32 = inner.next().unwrap().as_str().parse()?;
                Ok(Condition::Count(field_name, ComparisonOp::GreaterEqual, value))
            }
            "<=" => {
                let value: i32 = inner.next().unwrap().as_str().parse()?;
                Ok(Condition::Count(field_name, ComparisonOp::LessEqual, value))
            }
            _ => Ok(Condition::Checked(field_name))
        }
    } else {
        Ok(Condition::Checked(field_name))
    }
}

fn parse_field_value(pair: pest::iterators::Pair<Rule>) -> Result<String> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::quoted_string => parse_quoted_string(inner),
        Rule::word => Ok(inner.as_str().to_string()),
        _ => Ok(inner.as_str().to_string()),
    }
}

fn parse_conditional_body(pair: pest::iterators::Pair<Rule>) -> Result<Vec<Element>> {
    // The conditional body contains an indented body
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::body {
            return parse_body(inner);
        }
    }
    Ok(Vec::new())
}

fn parse_section(pair: pest::iterators::Pair<Rule>) -> Result<Option<Element>> {
    let section_title = pair.into_inner()
        .find(|p| p.as_rule() == Rule::section_title)
        .map(|p| p.as_str().trim().to_string())
        .unwrap_or_default();
    
    Ok(Some(Element::Group {
        label: section_title,
        elements: Vec::new(), // Sections don't contain elements in the current model
    }))
}

fn parse_message(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let mut has_prefix = false;
    let mut prefix = "";
    let mut text = String::new();
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::message_prefix => {
                has_prefix = true;
                prefix = inner.as_str();
            }
            Rule::text_value => {
                text = parse_text_value(inner)?;
            }
            _ => {}
        }
    }
    
    if has_prefix {
        let formatted = match prefix {
            ">" => format!("ℹ️ {}", text),
            "!" => format!("⚠️ {}", text),
            "?" => format!("❓ {}", text),
            "•" => format!("• {}", text),
            _ => text,
        };
        Ok(Element::Text(formatted))
    } else {
        Ok(Element::Text(text))
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
    let inner = pair.into_inner().next().unwrap();
    Ok(inner.as_str().to_string())
}

fn ensure_force_yield(elements: &mut Vec<Element>) {
    let has_buttons = elements.iter().any(|e| matches!(e, Element::Buttons(_)));
    
    if has_buttons {
        for element in elements {
            if let Element::Buttons(ref mut labels) = element {
                if !labels.contains(&"Force Yield".to_string()) {
                    labels.push("Force Yield".to_string());
                }
                break;
            }
        }
    }
}