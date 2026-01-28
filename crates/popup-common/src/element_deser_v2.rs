use crate::{Element, OptionValue};
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::ser::SerializeMap;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

// --- Helper Types for Polymorphic Fields ---

/// Represents `options` that can be:
/// - A list of strings/objects: `["A", "B", {"value": "C", "description": "..."}]`
/// - A comma-separated string: `"A, B, C"`
#[derive(Deserialize)]
#[serde(untagged)]
enum PolyOptions {
    List(Vec<OptionValue>),
    String(String),
}

impl From<PolyOptions> for Vec<OptionValue> {
    fn from(opts: PolyOptions) -> Self {
        match opts {
            PolyOptions::List(list) => list,
            PolyOptions::String(s) => s
                .split(',')
                .map(|opt| OptionValue::Simple(opt.trim().to_string()))
                .filter(|opt| !opt.value().is_empty())
                .collect(),
        }
    }
}

/// Represents children (reveals/branches) that can be:
/// - A list of elements: `[{"text": "Hi"}]`
/// - A single element object: `{"text": "Hi"}`
/// - A string (implicit text): `"Hi"`
#[derive(Deserialize)]
#[serde(untagged)]
enum PolyChildren {
    List(Vec<Element>),
    Single(Element),
    String(String),
}

impl From<PolyChildren> for Vec<Element> {
    fn from(pc: PolyChildren) -> Self {
        match pc {
            PolyChildren::List(list) => list,
            PolyChildren::Single(elem) => vec![elem],
            PolyChildren::String(s) => vec![Element::Text {
                text: s,
                id: None,
                when: None,
            }],
        }
    }
}

// --- ID Generation Logic ---

fn label_to_snake_case(label: &str) -> String {
    let mut result = String::new();
    let mut prev_separator = true; 
    let mut prev_upper = false;

    let chars: Vec<char> = label.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_alphanumeric() {
            let should_underscore = c.is_uppercase()
                && !result.is_empty()
                && !prev_separator
                && (!prev_upper || (i + 1 < chars.len() && chars[i + 1].is_lowercase()));

            if should_underscore {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
            prev_separator = false;
            prev_upper = c.is_uppercase();
        } else if c == ' ' || c == '-' || c == '_' {
            if !result.is_empty() && !prev_separator {
                result.push('_');
            }
            prev_separator = true;
            prev_upper = false;
        } else {
            prev_upper = false;
        }
    }
    result.trim_end_matches('_').to_string()
}

// --- Main Deserializer ---

struct ElementVisitor;

impl<'de> Visitor<'de> for ElementVisitor {
    type Value = Element;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a valid popup element object")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        // 1. Collect all fields into a Value Map first
        // We need to inspect keys to determine the variant (discriminator)
        // and also collect unknown keys for option-as-key children.
        let mut obj: serde_json::Map<String, Value> = serde_json::Map::new();
        
        while let Some((key, value)) = map.next_entry()? {
            obj.insert(key, value);
        }

        // 2. Identify the Variant
        if let Some(text_val) = obj.remove("text") {
            let text = text_val.as_str().ok_or_else(|| de::Error::custom("text must be string"))?.to_string();
            let id = obj.remove("id").and_then(|v| v.as_str().map(|s| s.to_string()));
            let when = obj.remove("when").and_then(|v| v.as_str().map(|s| s.to_string()));
            return Ok(Element::Text { text, id, when });
        }

        if let Some(md_val) = obj.remove("markdown") {
            let markdown = md_val.as_str().ok_or_else(|| de::Error::custom("markdown must be string"))?.to_string();
            let id = obj.remove("id").and_then(|v| v.as_str().map(|s| s.to_string()));
            let when = obj.remove("when").and_then(|v| v.as_str().map(|s| s.to_string()));
            return Ok(Element::Markdown { markdown, id, when });
        }

        if let Some(lbl_val) = obj.remove("slider") {
            let slider = lbl_val.as_str().ok_or_else(|| de::Error::custom("slider must be string"))?.to_string();
            let id = obj.remove("id").and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| label_to_snake_case(&slider));
            
            let min = obj.remove("min").and_then(|v| v.as_f64()).ok_or_else(|| de::Error::custom("missing min"))? as f32;
            let max = obj.remove("max").and_then(|v| v.as_f64()).ok_or_else(|| de::Error::custom("missing max"))? as f32;
            let default = obj.remove("default").and_then(|v| v.as_f64().map(|f| f as f32));
            let when = obj.remove("when").and_then(|v| v.as_str().map(|s| s.to_string()));

            return Ok(Element::Slider { slider, id, min, max, default, when });
        }

        if let Some(lbl_val) = obj.remove("check") {
            let check = lbl_val.as_str().ok_or_else(|| de::Error::custom("check must be string"))?.to_string();
            let id = obj.remove("id").and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| label_to_snake_case(&check));
            let default = obj.remove("default").and_then(|v| v.as_bool()).unwrap_or(false);
            let when = obj.remove("when").and_then(|v| v.as_str().map(|s| s.to_string()));
            
            let reveals = if let Some(rev_val) = obj.remove("reveals") {
                let pc: PolyChildren = serde_json::from_value(rev_val).map_err(de::Error::custom)?;
                pc.into()
            } else {
                Vec::new()
            };

            return Ok(Element::Check { check, id, default, reveals, when });
        }

        if let Some(lbl_val) = obj.remove("input") {
            let input = lbl_val.as_str().ok_or_else(|| de::Error::custom("input must be string"))?.to_string();
            let id = obj.remove("id").and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| label_to_snake_case(&input));
            let placeholder = obj.remove("placeholder").and_then(|v| v.as_str().map(|s| s.to_string()));
            let rows = obj.remove("rows").and_then(|v| v.as_u64().map(|u| u as u32));
            let when = obj.remove("when").and_then(|v| v.as_str().map(|s| s.to_string()));

            return Ok(Element::Input { input, id, placeholder, rows, when });
        }

        if let Some(lbl_val) = obj.remove("select") {
            let select = lbl_val.as_str().ok_or_else(|| de::Error::custom("select must be string"))?.to_string();
            let id = obj.remove("id").and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| label_to_snake_case(&select));
            
            let opts_val = obj.remove("options").ok_or_else(|| de::Error::custom("missing options"))?;
            let options: Vec<OptionValue> = serde_json::from_value::<PolyOptions>(opts_val)
                .map_err(de::Error::custom)?.into();
            
            let default = obj.remove("default").and_then(|v| v.as_str().map(|s| s.to_string()));
            let when = obj.remove("when").and_then(|v| v.as_str().map(|s| s.to_string()));

            let reveals = if let Some(rev_val) = obj.remove("reveals") {
                let pc: PolyChildren = serde_json::from_value(rev_val).map_err(de::Error::custom)?;
                pc.into()
            } else {
                Vec::new()
            };

            // Process option-as-key children (remaining fields)
            let mut option_children = HashMap::new();
            let valid_options: Vec<&str> = options.iter().map(|o| o.value()).collect();

            // Iterate over remaining keys in obj
            // Note: obj only contains keys we haven't removed yet
            for (key, val) in obj {
                if valid_options.contains(&key.as_str()) {
                    let children: Vec<Element> = serde_json::from_value::<PolyChildren>(val)
                        .map_err(de::Error::custom)?.into();
                    option_children.insert(key, children);
                }
            }

            return Ok(Element::Select { select, id, options, default, option_children, reveals, when });
        }

        if let Some(lbl_val) = obj.remove("multi") {
            let multi = lbl_val.as_str().ok_or_else(|| de::Error::custom("multi must be string"))?.to_string();
            let id = obj.remove("id").and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| label_to_snake_case(&multi));
            
            let opts_val = obj.remove("options").ok_or_else(|| de::Error::custom("missing options"))?;
            let options: Vec<OptionValue> = serde_json::from_value::<PolyOptions>(opts_val)
                .map_err(de::Error::custom)?.into();
            
            let when = obj.remove("when").and_then(|v| v.as_str().map(|s| s.to_string()));

            let reveals = if let Some(rev_val) = obj.remove("reveals") {
                let pc: PolyChildren = serde_json::from_value(rev_val).map_err(de::Error::custom)?;
                pc.into()
            } else {
                Vec::new()
            };

            // Process option-as-key children
            let mut option_children = HashMap::new();
            let valid_options: Vec<&str> = options.iter().map(|o| o.value()).collect();

            for (key, val) in obj {
                if valid_options.contains(&key.as_str()) {
                    let children: Vec<Element> = serde_json::from_value::<PolyChildren>(val)
                        .map_err(de::Error::custom)?.into();
                    option_children.insert(key, children);
                }
            }

            return Ok(Element::Multi { multi, id, options, option_children, reveals, when });
        }

        if let Some(lbl_val) = obj.remove("group") {
            let group = lbl_val.as_str().ok_or_else(|| de::Error::custom("group must be string"))?.to_string();
            let id = obj.remove("id").and_then(|v| v.as_str().map(|s| s.to_string()));
            let when = obj.remove("when").and_then(|v| v.as_str().map(|s| s.to_string()));
            
            let elems_val = obj.remove("elements").ok_or_else(|| de::Error::custom("group missing elements"))?;
            // Groups must have array elements, but let's be kind and use PolyChildren just in case single object is passed
            let elements: Vec<Element> = serde_json::from_value::<PolyChildren>(elems_val)
                .map_err(de::Error::custom)?.into();

            return Ok(Element::Group { group, id, elements, when });
        }

        Err(de::Error::custom("Unknown element type"))
    }
}

// Override Deserialize for Element to use our Visitor
impl<'de> Deserialize<'de> for Element {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(ElementVisitor)
    }
}

// Keep the serializer logic as is (unchanged from v1, it's fine)
impl Serialize for Element {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // ... (Copy existing serialize logic or just delegate to a struct if we want)
        // For now, let's just do a manual map serialization again to be safe and self-contained
        // (Assuming you want this file to be fully self-contained replacement)
        
        match self {
            Element::Text { text, id, when } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("text", text)?;
                if let Some(v) = id { map.serialize_entry("id", v)?; }
                if let Some(v) = when { map.serialize_entry("when", v)?; }
                map.end()
            }
            Element::Markdown { markdown, id, when } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("markdown", markdown)?;
                if let Some(v) = id { map.serialize_entry("id", v)?; }
                if let Some(v) = when { map.serialize_entry("when", v)?; }
                map.end()
            }
            Element::Slider { slider, id, min, max, default, when } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("slider", slider)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("min", min)?;
                map.serialize_entry("max", max)?;
                if let Some(v) = default { map.serialize_entry("default", v)?; }
                if let Some(v) = when { map.serialize_entry("when", v)?; }
                map.end()
            }
            Element::Check { check, id, default, reveals, when } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("check", check)?;
                map.serialize_entry("id", id)?;
                if *default { map.serialize_entry("default", default)?; }
                if !reveals.is_empty() { map.serialize_entry("reveals", reveals)?; }
                if let Some(v) = when { map.serialize_entry("when", v)?; }
                map.end()
            }
            Element::Input { input, id, placeholder, rows, when } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("input", input)?;
                map.serialize_entry("id", id)?;
                if let Some(v) = placeholder { map.serialize_entry("placeholder", v)?; }
                if let Some(v) = rows { map.serialize_entry("rows", v)?; }
                if let Some(v) = when { map.serialize_entry("when", v)?; }
                map.end()
            }
            Element::Multi { multi, id, options, option_children, reveals, when } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("multi", multi)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("options", options)?;
                for (k, v) in option_children { map.serialize_entry(k, v)?; }
                if !reveals.is_empty() { map.serialize_entry("reveals", reveals)?; }
                if let Some(v) = when { map.serialize_entry("when", v)?; }
                map.end()
            }
            Element::Select { select, id, options, default, option_children, reveals, when } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("select", select)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("options", options)?;
                if let Some(v) = default { map.serialize_entry("default", v)?; }
                for (k, v) in option_children { map.serialize_entry(k, v)?; }
                if !reveals.is_empty() { map.serialize_entry("reveals", reveals)?; }
                if let Some(v) = when { map.serialize_entry("when", v)?; }
                map.end()
            }
            Element::Group { group, id, elements, when } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("group", group)?;
                if let Some(v) = id { map.serialize_entry("id", v)?; }
                map.serialize_entry("elements", elements)?;
                if let Some(v) = when { map.serialize_entry("when", v)?; }
                map.end()
            }
        }
    }
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
            when: Some("@enabled".to_string()),
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Element = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, original);
    }

    #[test]
    fn test_deserialize_slider() {
        let json = r#"{"slider": "Volume", "id": "vol", "min": 0, "max": 100}"#;
        let elem: Element = serde_json::from_str(json).unwrap();
        match elem {
            Element::Slider {
                slider,
                id,
                min,
                max,
                ..
            } => {
                assert_eq!(slider, "Volume");
                assert_eq!(id, "vol");
                assert_eq!(min, 0.0);
                assert_eq!(max, 100.0);
            }
            _ => panic!("Expected Slider variant"),
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

    // Phase 6: Auto-ID generation tests

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

    // Phase 7: Polymorphic Ergonomics tests

    #[test]
    fn test_string_options_shorthand() {
        let json = r#"{
            "select": "Pick",
            "options": "A, B, C"
        }"#;
        let elem: Element = serde_json::from_str(json).unwrap();
        match elem {
            Element::Select { options, .. } => {
                assert_eq!(options.len(), 3);
                assert_eq!(options[0].value(), "A");
                assert_eq!(options[1].value(), "B");
                assert_eq!(options[2].value(), "C");
            }
            _ => panic!("Expected Choice variant"),
        }
    }

    #[test]
    fn test_polymorphic_children_string() {
        let json = r#"{
            "select": "Pick",
            "options": "A, B",
            "A": "You picked A"
        }"#;
        let elem: Element = serde_json::from_str(json).unwrap();
        match elem {
            Element::Select { option_children, .. } => {
                let children = option_children.get("A").unwrap();
                assert_eq!(children.len(), 1);
                match &children[0] {
                    Element::Text { text, .. } => assert_eq!(text, "You picked A"),
                    _ => panic!("Expected Text child"),
                }
            }
            _ => panic!("Expected Choice variant"),
        }
    }

    #[test]
    fn test_polymorphic_children_object() {
        let json = r#"{
            "select": "Pick",
            "options": "A, B",
            "B": {"check": "Confirm B"}
        }"#;
        let elem: Element = serde_json::from_str(json).unwrap();
        match elem {
            Element::Select { option_children, .. } => {
                let children = option_children.get("B").unwrap();
                assert_eq!(children.len(), 1);
                match &children[0] {
                    Element::Check { check, .. } => assert_eq!(check, "Confirm B"),
                    _ => panic!("Expected Check child"),
                }
            }
            _ => panic!("Expected Choice variant"),
        }
    }
}