use anyhow::Result;
use pest::Parser;
use pest_derive::Parser;

use crate::models::{Element, PopupDefinition, Condition, ComparisonOp};

// VIBES: Semantic error hints that guide toward valid patterns
fn get_semantic_error_hint(error_text: &str) -> String {
    if error_text.contains("boolean_value") {
        "💡 Boolean values: Use 'yes/no', 'true/false', 'enabled/disabled', or '✓/☐'".to_string()
    } else if error_text.contains("range_value") {
        "💡 Number ranges: Use format '0-100', '0..100', or '0 to 100' (optionally '= 50' for default)".to_string()
    } else if error_text.contains("choice_value") {
        "💡 Choice options: Use format 'Option1 | Option2 | Option3' (minimum 2 options required)".to_string()
    } else if error_text.contains("title_text") {
        "💡 Title format: Use plain text, '# Header', or 'confirm Question?'".to_string()
    } else if error_text.contains("button_text") {
        "💡 Button formats: Use '[OK | Cancel]', '→ Next', or 'Yes or No'".to_string()
    } else {
        "💡 Widget types: Sliders (0-100), Choices (A | B), Checkboxes (yes/no), Textboxes (@hint)".to_string()
    }
}

#[derive(Parser)]
#[grammar = "src/simple.pest"]
pub struct SimpleParser;

pub fn parse_popup_dsl(input: &str) -> Result<PopupDefinition> {
    // For backward compatibility, try to extract title from first line
    // if it looks like a title pattern
    let lines: Vec<&str> = input.lines().collect();
    
    if lines.is_empty() {
        return parse_popup_dsl_with_title("", None);
    }
    
    let first_line = lines[0].trim();
    
    // If first line is empty (whitespace only), don't treat as title
    if first_line.is_empty() {
        return parse_popup_dsl_with_title(input, None);
    }
    
    // Check if first line looks like a title
    let is_title = first_line.starts_with("confirm ") 
        || first_line.starts_with("# ")
        || first_line.starts_with("## ")
        || first_line.starts_with("### ")
        || (
            // If first line doesn't have widget syntax, treat as title
            !first_line.contains(':')
            && !first_line.starts_with('[')
            && !first_line.starts_with('!')
            && !first_line.starts_with('>')
            && !first_line.starts_with('?')
            && !first_line.starts_with('•')
            && !first_line.starts_with("→")
            && !first_line.contains(" or ")
        );
    
    if is_title {
        // Extract title and parse rest as body
        let mut title = first_line.to_string();
        
        // Strip confirm prefix if present
        if title.starts_with("confirm ") {
            title = title.strip_prefix("confirm ").unwrap().to_string();
        }
        
        // Strip markdown prefix if present
        if title.starts_with("# ") {
            title = title.strip_prefix("# ").unwrap().to_string();
        } else if title.starts_with("## ") {
            title = title.strip_prefix("## ").unwrap().to_string();
        } else if title.starts_with("### ") {
            title = title.strip_prefix("### ").unwrap().to_string();
        }
        
        // Add back question mark for confirm style
        if first_line.starts_with("confirm ") && !title.ends_with('?') {
            title.push('?');
        }
        
        let body = lines[1..].join("\n");
        parse_popup_dsl_with_title(&body, Some(title))
    } else {
        // No title detected, parse everything as elements
        parse_popup_dsl_with_title(input, None)
    }
}

pub fn parse_popup_dsl_with_title(input: &str, title: Option<String>) -> Result<PopupDefinition> {
    let pairs = SimpleParser::parse(Rule::popup, input)
        .map_err(|e| {
            // VIBES: Semantic error messages that teach the conceptual model
            let semantic_hint = get_semantic_error_hint(&e.to_string());
            let error_msg = format!("{}\n\n{}\n\nSimple syntax examples:\n\nBasic confirmation:\n  Yes or No\n\nSettings form:\n  Volume: 0-100          # 🎛️ Slider widget\n  Theme: Light | Dark    # 📋 Choice widget\n  [Save | Cancel]\n\nFor more examples, see the documentation.", e, semantic_hint);
            anyhow::anyhow!(error_msg)
        })?;
    
    let popup_pair = pairs.into_iter().next().unwrap();
    parse_popup(popup_pair, title)
}

fn parse_popup(pair: pest::iterators::Pair<Rule>, title: Option<String>) -> Result<PopupDefinition> {
    let mut elements = Vec::new();
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::element => {
                if let Some(elem) = parse_element(inner)? {
                    elements.push(elem);
                }
            }
            Rule::EOI => {}
            _ => {}
        }
    }
    
    let title = title.unwrap_or_else(|| "Popup".to_string());
    Ok(PopupDefinition { title, elements })
}


fn parse_element(pair: pest::iterators::Pair<Rule>) -> Result<Option<Element>> {
    let inner = pair.into_inner().next().unwrap();
    
    match inner.as_rule() {
        Rule::conditional => parse_conditional(inner).map(Some),
        Rule::labeled_item => parse_labeled_item(inner).map(Some),
        Rule::buttons => parse_buttons(inner).map(Some),
        Rule::message => parse_message(inner).map(Some),
        Rule::text_block => parse_text_block(inner).map(Some),
        _ => Ok(None),
    }
}


fn parse_text_block(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let mut lines = Vec::new();
    
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::text_line {
            lines.push(inner.as_str().trim().to_string());
        }
    }
    
    // Join multiple lines with newlines, or use single line as-is
    let text = if lines.len() == 1 {
        lines[0].clone()
    } else {
        lines.join("\n")
    };
    
    Ok(Element::Text(text))
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
    
    // Textbox pattern: @placeholder - check FIRST before other patterns with special chars
    if value.starts_with('@') {
        let placeholder = if value.len() > 1 {
            Some(value[1..].trim().to_string())
        } else {
            None
        };
        eprintln!("  -> Matched as Textbox");
        return Some(Element::Textbox {
            label: label.to_string(),
            placeholder,
            rows: None,
        });
    }
    
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
    
    // Multiselect pattern: [A, B, C] - check first to avoid comma confusion
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
    
    // Choice pattern: A | B | C, A, B, C, or A / B / C
    // But don't treat filenames or other text with commas as choices
    if value.contains(" | ") || value.contains(",") || value.contains("/") {
        let options: Vec<String> = if value.contains(" | ") {
            value.split(" | ").map(|s| s.trim().to_string()).collect()
        } else if value.contains(",") {
            // Only treat as choice if it looks like a proper choice list
            // Skip if it contains file extensions or other non-choice indicators
            if value.contains(".") || value.contains(" ") && !value.contains(", ") {
                return None; // Let it fall through to text
            }
            value.split(",").map(|s| s.trim().to_string()).collect()
        } else if value.contains("/") {
            // Only treat as choice if it doesn't look like a path
            if value.contains(".") || value.starts_with("/") || value.contains("\\") {
                return None; // Let it fall through to text
            }
            value.split("/").map(|s| s.trim().to_string()).collect()
        } else {
            vec![]
        };
        
        if options.len() > 1 {
            return Some(Element::Choice {
                label: label.to_string(),
                options,
            });
        }
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

fn parse_conditional(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    // For now, with atomic rule, parse the text manually
    let text = pair.as_str();
    
    // Extract condition from "[if CONDITION] {..."
    let cond_start = text.find("[if ").unwrap() + 4;
    let cond_end = text.find("] {").unwrap();
    let condition_text = &text[cond_start..cond_end];
    
    // Extract body from "...{ BODY }" - need to find matching closing brace
    let body_start = text.find("] {").unwrap() + 3;
    
    // Find the matching closing brace by counting depth
    let mut depth = 1;
    let mut body_end = body_start;
    let chars: Vec<char> = text[body_start..].chars().collect();
    
    for (i, ch) in chars.iter().enumerate() {
        if *ch == '{' {
            depth += 1;
        } else if *ch == '}' {
            depth -= 1;
            if depth == 0 {
                body_end = body_start + i;
                break;
            }
        }
    }
    
    let body_text = &text[body_start..body_end];
    
    // Parse condition
    let condition = parse_condition_text(condition_text.trim())?;
    
    // Parse body elements if not empty
    let mut elements = Vec::new();
    if !body_text.trim().is_empty() {
        // Process body text line by line, stripping leading whitespace
        let lines: Vec<&str> = body_text.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect();
        
        // Re-parse the cleaned body as a popup to get elements
        let cleaned_body = lines.join("\n");
        match SimpleParser::parse(Rule::popup, &format!("{}\n", cleaned_body)) {
            Ok(pairs) => {
                for pair in pairs {
                    if pair.as_rule() == Rule::popup {
                        for inner in pair.into_inner() {
                            if let Rule::element = inner.as_rule() {
                                if let Some(elem) = parse_element(inner)? {
                                    elements.push(elem);
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => {
                // If parsing fails, treat as plain text
                elements.push(Element::Text(cleaned_body));
            }
        }
    }
    
    Ok(Element::Conditional { condition, elements })
}

// Normalize labels for consistent matching
fn normalize_label(label: &str) -> String {
    label.trim()
         .trim_end_matches(':')
         .to_string()
}

fn parse_condition_text(condition_text: &str) -> Result<Condition> {
    // Parse condition patterns
    if condition_text.starts_with("not ") {
        let label = condition_text.strip_prefix("not ").unwrap().trim();
        Ok(Condition::Selected(normalize_label(label), "false".to_string()))
    } else if condition_text.contains(" has ") {
        let parts: Vec<&str> = condition_text.splitn(2, " has ").collect();
        if parts.len() == 2 {
            Ok(Condition::Selected(format!("{}:has", normalize_label(parts[0])), parts[1].trim().to_string()))
        } else {
            Ok(Condition::Checked(normalize_label(condition_text)))
        }
    // Check >= and <= before > and < to avoid wrong matches
    } else if condition_text.contains(" >= ") {
        let parts: Vec<&str> = condition_text.splitn(2, " >= ").collect();
        if parts.len() == 2 {
            let label = normalize_label(parts[0]);
            let value = parts[1].trim();
            if let Ok(num) = value.parse::<i32>() {
                Ok(Condition::Count(label, ComparisonOp::GreaterEqual, num))
            } else {
                Ok(Condition::Selected(label, value.to_string()))
            }
        } else {
            Ok(Condition::Checked(normalize_label(condition_text)))
        }
    } else if condition_text.contains(" <= ") {
        let parts: Vec<&str> = condition_text.splitn(2, " <= ").collect();
        if parts.len() == 2 {
            let label = normalize_label(parts[0]);
            let value = parts[1].trim();
            if let Ok(num) = value.parse::<i32>() {
                Ok(Condition::Count(label, ComparisonOp::LessEqual, num))
            } else {
                Ok(Condition::Selected(label, value.to_string()))
            }
        } else {
            Ok(Condition::Checked(normalize_label(condition_text)))
        }
    } else if condition_text.contains(" = ") {
        let parts: Vec<&str> = condition_text.splitn(2, " = ").collect();
        if parts.len() == 2 {
            let label = normalize_label(parts[0]);
            let value = parts[1].trim();
            if let Ok(num) = value.parse::<i32>() {
                Ok(Condition::Count(label, ComparisonOp::Equal, num))
            } else {
                Ok(Condition::Selected(label, value.to_string()))
            }
        } else {
            Ok(Condition::Checked(normalize_label(condition_text)))
        }
    } else if condition_text.contains(" > ") {
        let parts: Vec<&str> = condition_text.splitn(2, " > ").collect();
        if parts.len() == 2 {
            let label = normalize_label(parts[0]);
            let value = parts[1].trim();
            if let Ok(num) = value.parse::<i32>() {
                Ok(Condition::Count(label, ComparisonOp::Greater, num))
            } else {
                Ok(Condition::Selected(label, value.to_string()))
            }
        } else {
            Ok(Condition::Checked(normalize_label(condition_text)))
        }
    } else if condition_text.contains(" < ") {
        let parts: Vec<&str> = condition_text.splitn(2, " < ").collect();
        if parts.len() == 2 {
            let label = normalize_label(parts[0]);
            let value = parts[1].trim();
            if let Ok(num) = value.parse::<i32>() {
                Ok(Condition::Count(label, ComparisonOp::Less, num))
            } else {
                Ok(Condition::Selected(label, value.to_string()))
            }
        } else {
            Ok(Condition::Checked(normalize_label(condition_text)))
        }
    } else {
        // Simple condition - just a checkbox name
        Ok(Condition::Checked(normalize_label(condition_text)))
    }
}


