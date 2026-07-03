/// Custom serialization and deserialization for Element enum with element-as-key and option-as-key patterns
///
/// **Serialization**: Element variant name becomes JSON discriminator key
/// Example: Element::Text { text: "Hello", ... } → {"text": "Hello", ...}
///
/// **Deserialization**: The discriminator key selects the variant. Select and
/// Multi additionally accept their option values as direct keys mapping to
/// child element arrays (option-as-key branching).
///
/// Deserialization is strict: unknown keys are an error with a "did you mean"
/// suggestion. A typo'd branch key silently dropping an entire subtree is the
/// worst failure mode this interface can have, so we fail loudly instead.
use crate::validate::suggest;
use crate::{Element, OptionValue, SelectStyle};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;

/// Convert a label to snake_case for auto-generated IDs
/// Examples:
///   "Volume" -> "volume"
///   "Enable Feature" -> "enable_feature"
///   "What's the cause?" -> "whats_the_cause"
///   "CPU Usage" -> "cpu_usage"
fn label_to_snake_case(label: &str) -> String {
    let mut result = String::new();
    let mut prev_was_separator = true; // Start as true to not add underscore at beginning
    let mut prev_was_upper = false;

    let chars: Vec<char> = label.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_alphanumeric() {
            // Check if we should add underscore before this char:
            // - If current is uppercase and previous was lowercase (camelCase boundary)
            // - If current is uppercase and next is lowercase and we had uppercase before (HTTP -> h_t_t_p without this, HTTPServer -> http_server with this)
            let should_add_underscore = c.is_uppercase()
                && !result.is_empty()
                && !prev_was_separator
                && (!prev_was_upper || (i + 1 < chars.len() && chars[i + 1].is_lowercase()));

            if should_add_underscore {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
            prev_was_separator = false;
            prev_was_upper = c.is_uppercase();
        } else if c == ' ' || c == '-' || c == '_' {
            if !result.is_empty() && !prev_was_separator {
                result.push('_');
            }
            prev_was_separator = true;
            prev_was_upper = false;
        } else {
            // Skip other characters (punctuation, etc.) but don't treat as separator
            prev_was_upper = false;
        }
    }

    // Trim trailing underscores
    result.trim_end_matches('_').to_string()
}

/// Get ID from object, falling back to auto-generated from label
fn get_id_or_auto(obj: &serde_json::Map<String, Value>, label: &str) -> String {
    obj.get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| label_to_snake_case(label))
}

/// Reject unknown keys with an actionable message.
///
/// `branch_keys` holds option values that are legal as direct keys
/// (option-as-key branching on select/multi); empty for other elements.
fn check_unknown_keys<E: serde::de::Error>(
    obj: &serde_json::Map<String, Value>,
    kind: &str,
    label: &str,
    allowed: &[&str],
    branch_keys: &[&str],
) -> Result<(), E> {
    for key in obj.keys() {
        if allowed.contains(&key.as_str()) || branch_keys.contains(&key.as_str()) {
            continue;
        }
        let mut msg = format!(
            "{} \"{}\": unknown key \"{}\". Allowed keys: {}",
            kind,
            label,
            key,
            allowed.join(", ")
        );
        if !branch_keys.is_empty() {
            msg.push_str(&format!(
                ", or an option value as a branch key ({})",
                branch_keys.join(", ")
            ));
        }
        if let Some(s) = suggest(
            key,
            allowed.iter().copied().chain(branch_keys.iter().copied()),
        ) {
            msg.push_str(&format!(". Did you mean \"{}\"?", s));
        }
        return Err(serde::de::Error::custom(msg));
    }
    Ok(())
}

/// Custom Serialize for Element - produces element-as-key format
impl Serialize for Element {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Element::Text { text, id, when } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("text", text)?;
                if let Some(id_val) = id {
                    map.serialize_entry("id", id_val)?;
                }
                if let Some(when_val) = when {
                    map.serialize_entry("when", when_val)?;
                }
                map.end()
            }

            Element::Markdown { markdown, id, when } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("markdown", markdown)?;
                if let Some(id_val) = id {
                    map.serialize_entry("id", id_val)?;
                }
                if let Some(when_val) = when {
                    map.serialize_entry("when", when_val)?;
                }
                map.end()
            }

            Element::Slider {
                slider,
                id,
                min,
                max,
                default,
                step,
                when,
            } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("slider", slider)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("min", min)?;
                map.serialize_entry("max", max)?;
                if let Some(d) = default {
                    map.serialize_entry("default", d)?;
                }
                if let Some(s) = step {
                    map.serialize_entry("step", s)?;
                }
                if let Some(w) = when {
                    map.serialize_entry("when", w)?;
                }
                map.end()
            }

            Element::Check {
                check,
                id,
                default,
                reveals,
                when,
            } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("check", check)?;
                map.serialize_entry("id", id)?;
                if *default {
                    // Only serialize if true (false is default)
                    map.serialize_entry("default", default)?;
                }
                if !reveals.is_empty() {
                    map.serialize_entry("reveals", reveals)?;
                }
                if let Some(w) = when {
                    map.serialize_entry("when", w)?;
                }
                map.end()
            }

            Element::Input {
                input,
                id,
                placeholder,
                default,
                rows,
                when,
            } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("input", input)?;
                map.serialize_entry("id", id)?;
                if let Some(p) = placeholder {
                    map.serialize_entry("placeholder", p)?;
                }
                if let Some(d) = default {
                    map.serialize_entry("default", d)?;
                }
                if let Some(r) = rows {
                    map.serialize_entry("rows", r)?;
                }
                if let Some(w) = when {
                    map.serialize_entry("when", w)?;
                }
                map.end()
            }

            Element::Multi {
                multi,
                id,
                options,
                default,
                option_children,
                reveals,
                when,
            } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("multi", multi)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("options", options)?;
                if !default.is_empty() {
                    map.serialize_entry("default", default)?;
                }
                // Serialize option_children as direct keys (option-as-key pattern)
                for (option_key, children) in option_children {
                    map.serialize_entry(option_key, children)?;
                }
                if !reveals.is_empty() {
                    map.serialize_entry("reveals", reveals)?;
                }
                if let Some(w) = when {
                    map.serialize_entry("when", w)?;
                }
                map.end()
            }

            Element::Select {
                select,
                id,
                options,
                default,
                style,
                option_children,
                reveals,
                when,
            } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("select", select)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("options", options)?;
                if let Some(d) = default {
                    map.serialize_entry("default", d)?;
                }
                if let Some(s) = style {
                    map.serialize_entry("style", s)?;
                }
                // Serialize option_children as direct keys (option-as-key pattern)
                for (option_key, children) in option_children {
                    map.serialize_entry(option_key, children)?;
                }
                if !reveals.is_empty() {
                    map.serialize_entry("reveals", reveals)?;
                }
                if let Some(w) = when {
                    map.serialize_entry("when", w)?;
                }
                map.end()
            }

            Element::Rank {
                rank,
                id,
                options,
                when,
            } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("rank", rank)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("options", options)?;
                if let Some(w) = when {
                    map.serialize_entry("when", w)?;
                }
                map.end()
            }

            Element::Group {
                group,
                id,
                elements,
                when,
            } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("group", group)?;
                if let Some(id_val) = id {
                    map.serialize_entry("id", id_val)?;
                }
                map.serialize_entry("elements", elements)?;
                if let Some(w) = when {
                    map.serialize_entry("when", w)?;
                }
                map.end()
            }

            Element::Page {
                page,
                id,
                elements,
                when,
            } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("page", page)?;
                if let Some(id_val) = id {
                    map.serialize_entry("id", id_val)?;
                }
                map.serialize_entry("elements", elements)?;
                if let Some(w) = when {
                    map.serialize_entry("when", w)?;
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Element {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize to generic Value for inspection
        let value = Value::deserialize(deserializer)?;

        // Must be an object
        let obj = value
            .as_object()
            .ok_or_else(|| serde::de::Error::custom("Element must be a JSON object"))?;

        // Detect which variant by checking discriminator keys
        if obj.contains_key("text") {
            deserialize_text(obj)
        } else if obj.contains_key("markdown") {
            deserialize_markdown(obj)
        } else if obj.contains_key("slider") {
            deserialize_slider(obj)
        } else if obj.contains_key("check") {
            deserialize_checkbox(obj)
        } else if obj.contains_key("input") {
            deserialize_textbox(obj)
        } else if obj.contains_key("multi") {
            deserialize_multiselect(obj)
        } else if obj.contains_key("select") {
            deserialize_choice(obj)
        } else if obj.contains_key("rank") {
            deserialize_rank(obj)
        } else if obj.contains_key("group") {
            deserialize_group(obj)
        } else if obj.contains_key("page") {
            deserialize_page(obj)
        } else {
            let keys: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
            Err(serde::de::Error::custom(format!(
                "Unknown element type (keys present: {}) - every element needs exactly one of: text, markdown, slider, check, input, multi, select, rank, group, page",
                keys.join(", ")
            )))
        }
    }
}

fn get_when<E: serde::de::Error>(
    obj: &serde_json::Map<String, Value>,
) -> Result<Option<String>, E> {
    match obj.get("when") {
        None => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.clone())),
        Some(_) => Err(serde::de::Error::custom(
            "\"when\" must be a string condition expression",
        )),
    }
}

fn deserialize_text<E: serde::de::Error>(
    obj: &serde_json::Map<String, Value>,
) -> Result<Element, E> {
    let text = obj
        .get("text")
        .and_then(|v| v.as_str())
        .ok_or_else(|| serde::de::Error::custom("text field must be a string"))?
        .to_string();

    check_unknown_keys(obj, "text", &text, &["text", "id", "when"], &[])?;

    let id = obj
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(Element::Text {
        text,
        id,
        when: get_when(obj)?,
    })
}

fn deserialize_markdown<E: serde::de::Error>(
    obj: &serde_json::Map<String, Value>,
) -> Result<Element, E> {
    let markdown = obj
        .get("markdown")
        .and_then(|v| v.as_str())
        .ok_or_else(|| serde::de::Error::custom("markdown field must be a string"))?
        .to_string();

    check_unknown_keys(obj, "markdown", &markdown, &["markdown", "id", "when"], &[])?;

    let id = obj
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(Element::Markdown {
        markdown,
        id,
        when: get_when(obj)?,
    })
}

fn deserialize_slider<E: serde::de::Error>(
    obj: &serde_json::Map<String, Value>,
) -> Result<Element, E> {
    let slider = obj
        .get("slider")
        .and_then(|v| v.as_str())
        .ok_or_else(|| serde::de::Error::custom("slider field must be a string"))?
        .to_string();

    check_unknown_keys(
        obj,
        "slider",
        &slider,
        &["slider", "id", "min", "max", "default", "step", "when"],
        &[],
    )?;

    // Auto-generate ID from label if not provided
    let id = get_id_or_auto(obj, &slider);

    let min = obj
        .get("min")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| serde::de::Error::custom("slider must have min field (number)"))?
        as f32;

    let max = obj
        .get("max")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| serde::de::Error::custom("slider must have max field (number)"))?
        as f32;

    let default = obj
        .get("default")
        .and_then(|v| v.as_f64())
        .map(|f| f as f32);

    let step = obj.get("step").and_then(|v| v.as_f64()).map(|f| f as f32);

    Ok(Element::Slider {
        slider,
        id,
        min,
        max,
        default,
        step,
        when: get_when(obj)?,
    })
}

fn deserialize_checkbox<E: serde::de::Error>(
    obj: &serde_json::Map<String, Value>,
) -> Result<Element, E> {
    let check = obj
        .get("check")
        .and_then(|v| v.as_str())
        .ok_or_else(|| serde::de::Error::custom("check field must be a string"))?
        .to_string();

    check_unknown_keys(
        obj,
        "check",
        &check,
        &["check", "id", "default", "reveals", "when"],
        &[],
    )?;

    // Auto-generate ID from label if not provided
    let id = get_id_or_auto(obj, &check);

    let default = obj
        .get("default")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let reveals = obj
        .get("reveals")
        .map(|v| serde_json::from_value::<Vec<Element>>(v.clone()))
        .transpose()
        .map_err(serde::de::Error::custom)?
        .unwrap_or_default();

    Ok(Element::Check {
        check,
        id,
        default,
        reveals,
        when: get_when(obj)?,
    })
}

fn deserialize_textbox<E: serde::de::Error>(
    obj: &serde_json::Map<String, Value>,
) -> Result<Element, E> {
    let input = obj
        .get("input")
        .and_then(|v| v.as_str())
        .ok_or_else(|| serde::de::Error::custom("input field must be a string"))?
        .to_string();

    check_unknown_keys(
        obj,
        "input",
        &input,
        &["input", "id", "placeholder", "default", "rows", "when"],
        &[],
    )?;

    // Auto-generate ID from label if not provided
    let id = get_id_or_auto(obj, &input);

    let placeholder = obj
        .get("placeholder")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let default = obj
        .get("default")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let rows = obj.get("rows").and_then(|v| v.as_u64()).map(|n| n as u32);

    Ok(Element::Input {
        input,
        id,
        placeholder,
        default,
        rows,
        when: get_when(obj)?,
    })
}

fn parse_options<E: serde::de::Error>(
    obj: &serde_json::Map<String, Value>,
    kind: &str,
) -> Result<Vec<OptionValue>, E> {
    let options = obj
        .get("options")
        .ok_or_else(|| serde::de::Error::custom(format!("{} must have options field", kind)))?;
    serde_json::from_value::<Vec<OptionValue>>(options.clone()).map_err(serde::de::Error::custom)
}

/// Extract option-as-key children: keys matching option values map to child
/// element arrays shown when that option is selected
fn parse_option_children<E: serde::de::Error>(
    obj: &serde_json::Map<String, Value>,
    known_fields: &[&str],
    options: &[OptionValue],
) -> Result<HashMap<String, Vec<Element>>, E> {
    let option_values: Vec<&str> = options.iter().map(|o| o.value()).collect();
    let mut option_children = HashMap::new();

    for (key, value) in obj.iter() {
        if !known_fields.contains(&key.as_str()) && option_values.contains(&key.as_str()) {
            let children = serde_json::from_value::<Vec<Element>>(value.clone()).map_err(|e| {
                serde::de::Error::custom(format!("Invalid children for option '{}': {}", key, e))
            })?;
            option_children.insert(key.clone(), children);
        }
    }
    Ok(option_children)
}

fn deserialize_multiselect<E: serde::de::Error>(
    obj: &serde_json::Map<String, Value>,
) -> Result<Element, E> {
    let multi = obj
        .get("multi")
        .and_then(|v| v.as_str())
        .ok_or_else(|| serde::de::Error::custom("multi field must be a string"))?
        .to_string();

    // Auto-generate ID from label if not provided
    let id = get_id_or_auto(obj, &multi);

    let options = parse_options(obj, "multi")?;

    let known_fields = crate::validate::MULTI_FIELDS;
    let option_values: Vec<&str> = options.iter().map(|o| o.value()).collect();
    check_unknown_keys(obj, "multi", &multi, known_fields, &option_values)?;

    let reveals = obj
        .get("reveals")
        .map(|v| serde_json::from_value::<Vec<Element>>(v.clone()))
        .transpose()
        .map_err(serde::de::Error::custom)?
        .unwrap_or_default();

    // Pre-checked options: array of option texts, or a single string
    let default = match obj.get("default") {
        None => Vec::new(),
        Some(Value::String(s)) => vec![s.clone()],
        Some(Value::Array(_)) => serde_json::from_value::<Vec<String>>(obj["default"].clone())
            .map_err(|_| {
                serde::de::Error::custom(format!(
                    "multi \"{}\": default must be an array of option strings",
                    multi
                ))
            })?,
        Some(_) => {
            return Err(serde::de::Error::custom(format!(
                "multi \"{}\": default must be an array of option strings",
                multi
            )))
        }
    };

    let option_children = parse_option_children(obj, known_fields, &options)?;

    Ok(Element::Multi {
        multi,
        id,
        options,
        default,
        option_children,
        reveals,
        when: get_when(obj)?,
    })
}

fn deserialize_choice<E: serde::de::Error>(
    obj: &serde_json::Map<String, Value>,
) -> Result<Element, E> {
    let select = obj
        .get("select")
        .and_then(|v| v.as_str())
        .ok_or_else(|| serde::de::Error::custom("select field must be a string"))?
        .to_string();

    // Auto-generate ID from label if not provided
    let id = get_id_or_auto(obj, &select);

    let options = parse_options(obj, "select")?;

    let known_fields = crate::validate::SELECT_FIELDS;
    let option_values: Vec<&str> = options.iter().map(|o| o.value()).collect();
    check_unknown_keys(obj, "select", &select, known_fields, &option_values)?;

    let default = obj
        .get("default")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let style = match obj.get("style") {
        None => None,
        Some(v) => Some(
            serde_json::from_value::<SelectStyle>(v.clone()).map_err(|_| {
                serde::de::Error::custom(format!(
                    "select \"{}\": style must be \"radio\" or \"dropdown\", got {}",
                    select, v
                ))
            })?,
        ),
    };

    let reveals = obj
        .get("reveals")
        .map(|v| serde_json::from_value::<Vec<Element>>(v.clone()))
        .transpose()
        .map_err(serde::de::Error::custom)?
        .unwrap_or_default();

    let option_children = parse_option_children(obj, known_fields, &options)?;

    Ok(Element::Select {
        select,
        id,
        options,
        default,
        style,
        option_children,
        reveals,
        when: get_when(obj)?,
    })
}

fn deserialize_rank<E: serde::de::Error>(
    obj: &serde_json::Map<String, Value>,
) -> Result<Element, E> {
    let rank = obj
        .get("rank")
        .and_then(|v| v.as_str())
        .ok_or_else(|| serde::de::Error::custom("rank field must be a string"))?
        .to_string();

    check_unknown_keys(obj, "rank", &rank, &["rank", "id", "options", "when"], &[])?;

    let id = get_id_or_auto(obj, &rank);
    let options = parse_options(obj, "rank")?;

    Ok(Element::Rank {
        rank,
        id,
        options,
        when: get_when(obj)?,
    })
}

fn deserialize_group<E: serde::de::Error>(
    obj: &serde_json::Map<String, Value>,
) -> Result<Element, E> {
    let group = obj
        .get("group")
        .and_then(|v| v.as_str())
        .ok_or_else(|| serde::de::Error::custom("group field must be a string"))?
        .to_string();

    check_unknown_keys(
        obj,
        "group",
        &group,
        &["group", "id", "elements", "when"],
        &[],
    )?;

    let id = obj
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let elements = obj
        .get("elements")
        .ok_or_else(|| serde::de::Error::custom("group must have elements field"))?;
    let elements = serde_json::from_value::<Vec<Element>>(elements.clone())
        .map_err(serde::de::Error::custom)?;

    Ok(Element::Group {
        group,
        id,
        elements,
        when: get_when(obj)?,
    })
}

fn deserialize_page<E: serde::de::Error>(
    obj: &serde_json::Map<String, Value>,
) -> Result<Element, E> {
    let page = obj
        .get("page")
        .and_then(|v| v.as_str())
        .ok_or_else(|| serde::de::Error::custom("page field must be a string"))?
        .to_string();

    check_unknown_keys(obj, "page", &page, &["page", "id", "elements", "when"], &[])?;

    let id = obj
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let elements = obj
        .get("elements")
        .ok_or_else(|| serde::de::Error::custom("page must have elements field"))?;
    let elements = serde_json::from_value::<Vec<Element>>(elements.clone())
        .map_err(serde::de::Error::custom)?;

    Ok(Element::Page {
        page,
        id,
        elements,
        when: get_when(obj)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_text() {
        let elem = Element::Text {
            text: "Hello world".to_string(),
            id: Some("msg".to_string()),
            when: None,
        };
        let json = serde_json::to_string(&elem).unwrap();
        assert!(json.contains(r#""text":"Hello world"#));
        assert!(json.contains(r#""id":"msg"#));
        assert!(!json.contains("when")); // Should not serialize None
    }

    #[test]
    fn test_deserialize_text() {
        let json = r#"{"text": "Hello world"}"#;
        let elem: Element = serde_json::from_str(json).unwrap();
        match elem {
            Element::Text { text, .. } => assert_eq!(text, "Hello world"),
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_roundtrip_text() {
        let original = Element::Text {
            text: "Test".to_string(),
            id: None,
            when: Some("enabled".to_string()),
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Element = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, original);
    }

    #[test]
    fn test_deserialize_slider() {
        let json = r#"{"slider": "Volume", "id": "vol", "min": 0, "max": 100, "step": 5}"#;
        let elem: Element = serde_json::from_str(json).unwrap();
        match elem {
            Element::Slider {
                slider,
                id,
                min,
                max,
                step,
                ..
            } => {
                assert_eq!(slider, "Volume");
                assert_eq!(id, "vol");
                assert_eq!(min, 0.0);
                assert_eq!(max, 100.0);
                assert_eq!(step, Some(5.0));
            }
            _ => panic!("Expected Slider variant"),
        }
    }

    #[test]
    fn test_deserialize_input_with_default() {
        let json = r#"{"input": "Name", "default": "proposed value"}"#;
        let elem: Element = serde_json::from_str(json).unwrap();
        match elem {
            Element::Input { default, .. } => {
                assert_eq!(default, Some("proposed value".to_string()));
            }
            _ => panic!("Expected Input variant"),
        }
    }

    #[test]
    fn test_deserialize_choice_with_option_children() {
        let json = r#"{
            "select": "Theme",
            "id": "theme",
            "options": ["Light", "Dark"],
            "Dark": [
                {"text": "Dark mode settings"}
            ]
        }"#;
        let elem: Element = serde_json::from_str(json).unwrap();
        match elem {
            Element::Select {
                select,
                id,
                options,
                option_children,
                ..
            } => {
                assert_eq!(select, "Theme");
                assert_eq!(id, "theme");
                assert_eq!(options.len(), 2);
                assert_eq!(options[0].value(), "Light");
                assert_eq!(options[1].value(), "Dark");
                assert!(option_children.contains_key("Dark"));
                assert_eq!(option_children.get("Dark").unwrap().len(), 1);
            }
            _ => panic!("Expected Choice variant"),
        }
    }

    #[test]
    fn test_deserialize_select_style() {
        let json = r#"{"select": "Theme", "options": ["A", "B"], "style": "radio"}"#;
        let elem: Element = serde_json::from_str(json).unwrap();
        match elem {
            Element::Select { style, .. } => assert_eq!(style, Some(SelectStyle::Radio)),
            _ => panic!("Expected Select variant"),
        }

        let json = r#"{"select": "Theme", "options": ["A", "B"], "style": "fancy"}"#;
        let err = serde_json::from_str::<Element>(json)
            .unwrap_err()
            .to_string();
        assert!(err.contains("radio"), "unexpected error: {}", err);
    }

    #[test]
    fn test_typod_branch_key_is_error_with_suggestion() {
        // "Drak" is not an option - previously this subtree silently vanished
        let json = r#"{
            "select": "Theme",
            "options": ["Light", "Dark"],
            "Drak": [{"text": "Dark mode settings"}]
        }"#;
        let err = serde_json::from_str::<Element>(json)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("unknown key \"Drak\""),
            "unexpected error: {}",
            err
        );
        assert!(
            err.contains("Did you mean \"Dark\"?"),
            "unexpected error: {}",
            err
        );
    }

    #[test]
    fn test_unknown_field_is_error() {
        let json = r#"{"slider": "Volume", "min": 0, "max": 100, "reveals": []}"#;
        let err = serde_json::from_str::<Element>(json)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("unknown key \"reveals\""),
            "unexpected error: {}",
            err
        );

        let json = r#"{"check": "Enable", "revales": []}"#;
        let err = serde_json::from_str::<Element>(json)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("Did you mean \"reveals\"?"),
            "unexpected error: {}",
            err
        );
    }

    #[test]
    fn test_deserialize_choice_with_option_descriptions() {
        let json = r#"{
            "select": "Approach",
            "id": "approach",
            "options": [
                "Simple",
                {"value": "Advanced", "description": "More control but complex"},
                {"value": "Expert", "because": "Full power for experts"}
            ]
        }"#;
        let elem: Element = serde_json::from_str(json).unwrap();
        match elem {
            Element::Select { options, .. } => {
                assert_eq!(options.len(), 3);
                assert_eq!(options[0].value(), "Simple");
                assert_eq!(options[0].description(), None);
                assert_eq!(options[1].value(), "Advanced");
                assert_eq!(options[1].description(), Some("More control but complex"));
                assert_eq!(options[2].value(), "Expert");
                assert_eq!(options[2].description(), Some("Full power for experts"));
                // "because" alias
            }
            _ => panic!("Expected Choice variant"),
        }
    }

    #[test]
    fn test_deserialize_multi_with_defaults() {
        let json = r#"{"multi": "Features", "options": ["A", "B", "C"], "default": ["A", "C"]}"#;
        let elem: Element = serde_json::from_str(json).unwrap();
        match elem {
            Element::Multi { default, .. } => {
                assert_eq!(default, vec!["A".to_string(), "C".to_string()])
            }
            _ => panic!("Expected Multi variant"),
        }

        // Single string accepted as a one-element default
        let json = r#"{"multi": "Features", "options": ["A", "B"], "default": "B"}"#;
        let elem: Element = serde_json::from_str(json).unwrap();
        match elem {
            Element::Multi { default, .. } => assert_eq!(default, vec!["B".to_string()]),
            _ => panic!("Expected Multi variant"),
        }

        // Non-string default is a clear error
        let json = r#"{"multi": "Features", "options": ["A"], "default": 3}"#;
        let err = serde_json::from_str::<Element>(json)
            .unwrap_err()
            .to_string();
        assert!(err.contains("array of option strings"), "got: {}", err);
    }

    #[test]
    fn test_deserialize_multiselect_with_option_children() {
        let json = r#"{
            "multi": "Features",
            "id": "features",
            "options": ["Basic", "Advanced"],
            "Advanced": [
                {"slider": "Level", "id": "level", "min": 1, "max": 10}
            ]
        }"#;
        let elem: Element = serde_json::from_str(json).unwrap();
        match elem {
            Element::Multi {
                multi,
                id,
                options,
                option_children,
                ..
            } => {
                assert_eq!(multi, "Features");
                assert_eq!(id, "features");
                assert_eq!(options.len(), 2);
                assert_eq!(options[0].value(), "Basic");
                assert_eq!(options[1].value(), "Advanced");
                assert!(option_children.contains_key("Advanced"));
                assert_eq!(option_children.get("Advanced").unwrap().len(), 1);
            }
            _ => panic!("Expected Multiselect variant"),
        }
    }

    #[test]
    fn test_serialize_slider() {
        let elem = Element::Slider {
            slider: "Volume".to_string(),
            id: "vol".to_string(),
            min: 0.0,
            max: 100.0,
            default: Some(75.0),
            step: None,
            when: None,
        };
        let json = serde_json::to_value(&elem).unwrap();
        assert_eq!(json["slider"], "Volume");
        assert_eq!(json["id"], "vol");
        assert_eq!(json["default"], 75.0);
    }

    #[test]
    fn test_roundtrip_choice_with_option_children() {
        let mut option_children = HashMap::new();
        option_children.insert(
            "Dark".to_string(),
            vec![Element::Slider {
                slider: "Brightness".to_string(),
                id: "brightness".to_string(),
                min: 0.0,
                max: 100.0,
                default: Some(50.0),
                step: None,
                when: None,
            }],
        );

        let original = Element::Select {
            select: "Theme".to_string(),
            id: "theme".to_string(),
            options: vec![
                OptionValue::Simple("Light".to_string()),
                OptionValue::Simple("Dark".to_string()),
            ],
            default: Some("Dark".to_string()),
            style: Some(SelectStyle::Radio),
            option_children,
            reveals: vec![],
            when: None,
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Element = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, original);
    }

    #[test]
    fn test_serialize_option_children_as_direct_keys() {
        let mut option_children = HashMap::new();
        option_children.insert(
            "Advanced".to_string(),
            vec![Element::Text {
                text: "Advanced mode".to_string(),
                id: None,
                when: None,
            }],
        );

        let elem = Element::Select {
            select: "Mode".to_string(),
            id: "mode".to_string(),
            options: vec![
                OptionValue::Simple("Basic".to_string()),
                OptionValue::Simple("Advanced".to_string()),
            ],
            default: None,
            style: None,
            option_children,
            reveals: vec![],
            when: None,
        };

        let json = serde_json::to_value(&elem).unwrap();

        // Verify option-as-key: "Advanced" should be a direct key in JSON
        assert!(json.get("Advanced").is_some());
        assert!(json["Advanced"].is_array());
        assert_eq!(json["Advanced"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_rank_roundtrip_and_strict() {
        let json = r#"{"rank": "Priorities", "options": ["A", "B"]}"#;
        let elem: Element = serde_json::from_str(json).unwrap();
        match &elem {
            Element::Rank {
                rank, id, options, ..
            } => {
                assert_eq!(rank, "Priorities");
                assert_eq!(id, "priorities"); // auto-derived
                assert_eq!(options.len(), 2);
            }
            _ => panic!("Expected Rank variant"),
        }
        let roundtripped: Element =
            serde_json::from_str(&serde_json::to_string(&elem).unwrap()).unwrap();
        assert_eq!(roundtripped, elem);

        // Rank keeps the schema lean: no reveals/branches
        let json = r#"{"rank": "Priorities", "options": ["A"], "reveals": []}"#;
        let err = serde_json::from_str::<Element>(json)
            .unwrap_err()
            .to_string();
        assert!(err.contains("unknown key \"reveals\""), "got: {}", err);
    }

    #[test]
    fn test_page_roundtrip() {
        let json = r#"{
            "page": "Basics",
            "elements": [{"input": "Name", "id": "name"}],
            "when": "advanced"
        }"#;
        let elem: Element = serde_json::from_str(json).unwrap();
        match &elem {
            Element::Page {
                page,
                elements,
                when,
                ..
            } => {
                assert_eq!(page, "Basics");
                assert_eq!(elements.len(), 1);
                assert_eq!(when.as_deref(), Some("advanced"));
            }
            _ => panic!("Expected Page variant"),
        }
        let roundtripped: Element =
            serde_json::from_str(&serde_json::to_string(&elem).unwrap()).unwrap();
        assert_eq!(roundtripped, elem);

        // Typo'd key errors with suggestion
        let json = r#"{"page": "Basics", "element": []}"#;
        let err = serde_json::from_str::<Element>(json)
            .unwrap_err()
            .to_string();
        assert!(err.contains("Did you mean \"elements\"?"), "got: {}", err);
    }

    #[test]
    fn test_roundtrip_all_variants() {
        let elements = vec![
            Element::Text {
                text: "Hello".to_string(),
                id: Some("msg".to_string()),
                when: None,
            },
            Element::Markdown {
                markdown: "## Header\n- **Bold** item\n- *Italic* item".to_string(),
                id: Some("content".to_string()),
                when: None,
            },
            Element::Slider {
                slider: "Volume".to_string(),
                id: "vol".to_string(),
                min: 0.0,
                max: 100.0,
                default: None,
                step: Some(0.5),
                when: None,
            },
            Element::Check {
                check: "Enable".to_string(),
                id: "enabled".to_string(),
                default: true,
                reveals: vec![],
                when: None,
            },
            Element::Input {
                input: "Name".to_string(),
                id: "name".to_string(),
                placeholder: Some("Enter name".to_string()),
                default: Some("Ada".to_string()),
                rows: Some(3),
                when: None,
            },
            Element::Multi {
                multi: "Options".to_string(),
                id: "opts".to_string(),
                options: vec![
                    OptionValue::Simple("A".to_string()),
                    OptionValue::Simple("B".to_string()),
                ],
                default: vec!["B".to_string()],
                option_children: HashMap::new(),
                reveals: vec![],
                when: None,
            },
            Element::Group {
                group: "Settings".to_string(),
                id: None,
                elements: vec![],
                when: None,
            },
        ];

        for original in elements {
            let json = serde_json::to_string(&original).unwrap();
            let deserialized: Element = serde_json::from_str(&json).unwrap();
            assert_eq!(
                deserialized, original,
                "Round-trip failed for {:?}",
                original
            );
        }
    }

    // Auto-ID generation tests

    #[test]
    fn test_label_to_snake_case() {
        // Simple cases
        assert_eq!(label_to_snake_case("Volume"), "volume");
        assert_eq!(label_to_snake_case("Enable Feature"), "enable_feature");
        assert_eq!(label_to_snake_case("CPU Usage"), "cpu_usage"); // Acronym + space
        assert_eq!(label_to_snake_case("HTTPServer"), "http_server"); // Acronym then word
        assert_eq!(label_to_snake_case("What's the cause?"), "whats_the_cause");
        assert_eq!(
            label_to_snake_case("Debug Level (1-10)"),
            "debug_level_1_10"
        );
        assert_eq!(label_to_snake_case("EnableDebug"), "enable_debug"); // CamelCase
        assert_eq!(label_to_snake_case("my-option"), "my_option"); // Dash separator
        assert_eq!(label_to_snake_case("  Spaced  Out  "), "spaced_out"); // Multiple spaces
        assert_eq!(label_to_snake_case("getHTTPResponse"), "get_http_response");
        // Mixed
    }

    #[test]
    fn test_auto_id_slider() {
        let json = r#"{"slider": "Volume", "min": 0, "max": 100}"#;
        let elem: Element = serde_json::from_str(json).unwrap();
        match elem {
            Element::Slider { id, .. } => assert_eq!(id, "volume"),
            _ => panic!("Expected Slider variant"),
        }

        // With explicit id
        let json = r#"{"slider": "Volume", "id": "vol", "min": 0, "max": 100}"#;
        let elem: Element = serde_json::from_str(json).unwrap();
        match elem {
            Element::Slider { id, .. } => assert_eq!(id, "vol"),
            _ => panic!("Expected Slider variant"),
        }
    }

    #[test]
    fn test_auto_id_checkbox() {
        let json = r#"{"check": "Enable Feature"}"#;
        let elem: Element = serde_json::from_str(json).unwrap();
        match elem {
            Element::Check { id, .. } => assert_eq!(id, "enable_feature"),
            _ => panic!("Expected Checkbox variant"),
        }
    }

    #[test]
    fn test_auto_id_textbox() {
        let json = r#"{"input": "User Name"}"#;
        let elem: Element = serde_json::from_str(json).unwrap();
        match elem {
            Element::Input { id, .. } => assert_eq!(id, "user_name"),
            _ => panic!("Expected Textbox variant"),
        }
    }

    #[test]
    fn test_auto_id_choice() {
        let json = r#"{"select": "Color Theme", "options": ["Light", "Dark"]}"#;
        let elem: Element = serde_json::from_str(json).unwrap();
        match elem {
            Element::Select { id, .. } => assert_eq!(id, "color_theme"),
            _ => panic!("Expected Choice variant"),
        }
    }

    #[test]
    fn test_auto_id_multiselect() {
        let json = r#"{"multi": "Selected Features", "options": ["A", "B", "C"]}"#;
        let elem: Element = serde_json::from_str(json).unwrap();
        match elem {
            Element::Multi { id, .. } => assert_eq!(id, "selected_features"),
            _ => panic!("Expected Multiselect variant"),
        }
    }

    #[test]
    fn test_auto_id_complex_labels() {
        // Test various edge cases
        let json = r#"{"check": "What's happening?"}"#;
        let elem: Element = serde_json::from_str(json).unwrap();
        match elem {
            Element::Check { id, .. } => assert_eq!(id, "whats_happening"),
            _ => panic!("Expected Checkbox variant"),
        }

        let json = r#"{"slider": "Level (1-10)", "min": 1, "max": 10}"#;
        let elem: Element = serde_json::from_str(json).unwrap();
        match elem {
            Element::Slider { id, .. } => assert_eq!(id, "level_1_10"),
            _ => panic!("Expected Slider variant"),
        }
    }
}
