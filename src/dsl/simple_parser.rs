use anyhow::Result;
use pest::Parser;
use pest_derive::Parser;

use crate::models::{Element, PopupDefinition};

#[derive(Parser)]
#[grammar = "src/simple.pest"]
pub struct SimpleParser;

pub fn parse_popup_dsl(input: &str) -> Result<PopupDefinition> {
    let pairs = SimpleParser::parse(Rule::popup, input)
        .map_err(|e| {
            let error_msg = format!("{}\n\nSimple syntax examples:\n\nBasic confirmation:\n  confirm Delete file?\n  Yes or No\n\nSettings form:\n  Settings\n  Volume: 0-100\n  Theme: Light | Dark\n  [Save | Cancel]\n\nFor more examples, see the documentation.", e);
            anyhow::anyhow!(error_msg)
        })?;
    
    let popup_pair = pairs.into_iter().next().unwrap();
    parse_popup(popup_pair)
}

fn parse_popup(pair: pest::iterators::Pair<Rule>) -> Result<PopupDefinition> {
    let mut title = String::new();
    let mut elements = Vec::new();
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::title => {
                title = parse_title(inner)?;
            }
            Rule::element => {
                if let Some(element) = parse_element(inner)? {
                    elements.push(element);
                }
            }
            Rule::EOI => {}
            _ => {}
        }
    }
    
    // Add Force Yield button if there are any buttons
    ensure_force_yield(&mut elements);
    
    Ok(PopupDefinition { title, elements })
}

fn parse_title(pair: pest::iterators::Pair<Rule>) -> Result<String> {
    // Skip "confirm" if present and get the text
    let text = pair.into_inner()
        .find(|p| p.as_rule() == Rule::text_line)
        .map(|p| p.as_str().trim().to_string())
        .unwrap_or_default();
    Ok(text)
}

fn parse_element(pair: pest::iterators::Pair<Rule>) -> Result<Option<Element>> {
    let inner = pair.into_inner().next().unwrap();
    
    match inner.as_rule() {
        Rule::labeled_item => parse_labeled_item(inner).map(Some),
        Rule::buttons => parse_buttons(inner).map(Some),
        Rule::message => parse_message(inner).map(Some),
        Rule::text_line => Ok(Some(Element::Text(inner.as_str().trim().to_string()))),
        _ => Ok(None),
    }
}

fn parse_labeled_item(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let mut inner = pair.into_inner();
    let label = inner.next().unwrap().as_str().trim().to_string();
    let value = inner.next().unwrap().as_str().trim();
    
    // Intelligently determine if this is a widget or just text
    if let Some(element) = try_parse_widget(&label, value) {
        Ok(element)
    } else {
        // Just display as text
        Ok(Element::Text(format!("{}: {}", label, value)))
    }
}

fn try_parse_widget(label: &str, value: &str) -> Option<Element> {
    // Try to parse as different widget types
    
    // Range pattern: 0-100, 0..100, 0 to 100
    if let Some((min, max, default)) = parse_range_pattern(value) {
        return Some(Element::Slider { 
            label: label.to_string(), 
            min, 
            max, 
            default 
        });
    }
    
    // Boolean pattern: yes, no, true, false, ✓, ☐, etc.
    if let Some(checked) = parse_bool_pattern(value) {
        return Some(Element::Checkbox {
            label: label.to_string(),
            default: checked,
        });
    }
    
    // Choice pattern: A | B | C
    if value.contains(" | ") {
        let options: Vec<String> = value.split(" | ")
            .map(|s| s.trim().to_string())
            .collect();
        if options.len() > 1 {
            return Some(Element::Choice {
                label: label.to_string(),
                options,
            });
        }
    }
    
    // Multiselect pattern: [A, B, C]
    if value.starts_with('[') && value.ends_with(']') {
        let inner = &value[1..value.len()-1];
        let options: Vec<String> = inner.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !options.is_empty() {
            return Some(Element::Multiselect {
                label: label.to_string(),
                options,
            });
        }
    }
    
    // Textbox pattern: @placeholder
    if value.starts_with('@') {
        let placeholder = if value.len() > 1 {
            Some(value[1..].trim().to_string())
        } else {
            None
        };
        return Some(Element::Textbox {
            label: label.to_string(),
            placeholder,
            rows: None,
        });
    }
    
    // Not a recognized widget pattern
    None
}

fn parse_range_pattern(value: &str) -> Option<(f32, f32, f32)> {
    // Try different range formats
    let patterns = [
        (r"^(\d+(?:\.\d+)?)\s*-\s*(\d+(?:\.\d+)?)\s*(?:=\s*(\d+(?:\.\d+)?))?$", "-"),
        (r"^(\d+(?:\.\d+)?)\s*\.\.\s*(\d+(?:\.\d+)?)\s*(?:=\s*(\d+(?:\.\d+)?))?$", ".."),
        (r"^(\d+(?:\.\d+)?)\s+to\s+(\d+(?:\.\d+)?)\s*(?:=\s*(\d+(?:\.\d+)?))?$", "to"),
    ];
    
    for (pattern, _sep) in patterns {
        let re = regex::Regex::new(pattern).ok()?;
        if let Some(caps) = re.captures(value) {
            let min: f32 = caps.get(1)?.as_str().parse().ok()?;
            let max: f32 = caps.get(2)?.as_str().parse().ok()?;
            let default = caps.get(3)
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or((min + max) / 2.0);
            return Some((min, max, default));
        }
    }
    
    None
}

fn parse_bool_pattern(value: &str) -> Option<bool> {
    match value.to_lowercase().as_str() {
        "yes" | "true" | "on" | "enabled" | "checked" => Some(true),
        "no" | "false" | "off" | "disabled" | "unchecked" => Some(false),
        _ => match value {
            "✓" | "☑" | "[x]" | "[X]" | "(*)" => Some(true),
            "☐" | "☒" | "[ ]" | "( )" => Some(false),
            _ => None,
        }
    }
}

fn parse_buttons(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let inner = pair.into_inner().next().unwrap();
    let mut labels = Vec::new();
    
    match inner.as_rule() {
        Rule::bracket_buttons => {
            for child in inner.into_inner() {
                if child.as_rule() == Rule::button_text {
                    labels.push(child.as_str().trim().to_string());
                }
            }
        }
        Rule::arrow_button => {
            for child in inner.into_inner() {
                if child.as_rule() == Rule::button_text {
                    labels.push(child.as_str().trim().to_string());
                }
            }
        }
        Rule::or_buttons => {
            for child in inner.into_inner() {
                if child.as_rule() == Rule::button_text {
                    labels.push(child.as_str().trim().to_string());
                }
            }
        }
        _ => {}
    }
    
    Ok(Element::Buttons(labels))
}

fn parse_message(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let mut prefix = None;
    let mut text = String::new();
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::message_prefix => {
                prefix = Some(inner.as_str());
            }
            Rule::text_line => {
                text = inner.as_str().trim().to_string();
            }
            _ => {}
        }
    }
    
    let formatted = match prefix {
        Some(">") => format!("ℹ️ {}", text),
        Some("!") => format!("⚠️ {}", text),
        Some("?") => format!("❓ {}", text),
        Some("•") => format!("• {}", text),
        _ => text,
    };
    
    Ok(Element::Text(formatted))
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