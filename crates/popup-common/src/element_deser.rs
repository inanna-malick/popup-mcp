/// Custom serialization and deserialization for Element enum with element-as-key and option-as-key patterns
///
/// **Serialization**: Element variant name becomes JSON discriminator key
/// Example: Element::Text { text: "Hello", ... } → {"text": "Hello", ...}
///
/// **Deserialization Challenge**: Choice and Multiselect have HashMap<String, Vec<Element>>
/// where keys are arbitrary option values. Standard serde(flatten) can't distinguish
/// between known fields and unknown option keys.
///
/// **Solution**: Manual Serialize/Deserialize impls that:
/// - Serialize: Use variant field as discriminator, flatten option_children as direct keys
/// - Deserialize: Extract known fields first, treat remaining matching option keys as children

use crate::Element;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::ser::SerializeMap;
use serde_json::Value;
use std::collections::HashMap;

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

            Element::Slider { slider, id, min, max, default, reveals, when } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("slider", slider)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("min", min)?;
                map.serialize_entry("max", max)?;
                if let Some(d) = default {
                    map.serialize_entry("default", d)?;
                }
                if !reveals.is_empty() {
                    map.serialize_entry("reveals", reveals)?;
                }
                if let Some(w) = when {
                    map.serialize_entry("when", w)?;
                }
                map.end()
            }

            Element::Checkbox { checkbox, id, default, reveals, when } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("checkbox", checkbox)?;
                map.serialize_entry("id", id)?;
                if *default {  // Only serialize if true (false is default)
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

            Element::Textbox { textbox, id, placeholder, rows, reveals, when } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("textbox", textbox)?;
                map.serialize_entry("id", id)?;
                if let Some(p) = placeholder {
                    map.serialize_entry("placeholder", p)?;
                }
                if let Some(r) = rows {
                    map.serialize_entry("rows", r)?;
                }
                if !reveals.is_empty() {
                    map.serialize_entry("reveals", reveals)?;
                }
                if let Some(w) = when {
                    map.serialize_entry("when", w)?;
                }
                map.end()
            }

            Element::Multiselect { multiselect, id, options, option_children, reveals, when } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("multiselect", multiselect)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("options", options)?;
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

            Element::Choice { choice, id, options, default, option_children, reveals, when } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("choice", choice)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("options", options)?;
                if let Some(d) = default {
                    map.serialize_entry("default", d)?;
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

            Element::Group { group, id, elements, when } => {
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
        }
    }
}

//
// Approach:
// 1. Deserialize to generic Value first
// 2. Check which discriminator key is present (text, slider, checkbox, etc.)
// 3. Extract known fields for that variant
// 4. For Choice/Multiselect: remaining keys become option_children HashMap
// 5. Reconstruct Element enum variant with extracted data
//
// Example JSON for Choice with option-as-key:
// {
//   "choice": "Theme",
//   "id": "theme",
//   "options": ["Light", "Dark"],
//   "Dark": [  // <-- option-as-key: "Dark" maps to nested elements
//     {"slider": "Brightness", "id": "brightness", "min": 0, "max": 100}
//   ]
// }

impl<'de> Deserialize<'de> for Element {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize to generic Value for inspection
        let value = Value::deserialize(deserializer)?;

        // Must be an object
        let obj = value.as_object()
            .ok_or_else(|| serde::de::Error::custom("Element must be a JSON object"))?;

        // Detect which variant by checking discriminator keys
        if obj.contains_key("text") {
            deserialize_text(obj)
        } else if obj.contains_key("slider") {
            deserialize_slider(obj)
        } else if obj.contains_key("checkbox") {
            deserialize_checkbox(obj)
        } else if obj.contains_key("textbox") {
            deserialize_textbox(obj)
        } else if obj.contains_key("multiselect") {
            deserialize_multiselect(obj)
        } else if obj.contains_key("choice") {
            deserialize_choice(obj)
        } else if obj.contains_key("group") {
            deserialize_group(obj)
        } else {
            Err(serde::de::Error::custom("Unknown element type - must have one of: text, slider, checkbox, textbox, multiselect, choice, group"))
        }
    }
}

fn deserialize_text<E: serde::de::Error>(obj: &serde_json::Map<String, Value>) -> Result<Element, E> {
    let text = obj.get("text")
        .and_then(|v| v.as_str())
        .ok_or_else(|| serde::de::Error::custom("text field must be a string"))?
        .to_string();

    let id = obj.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
    let when = obj.get("when").and_then(|v| v.as_str()).map(|s| s.to_string());

    Ok(Element::Text { text, id, when })
}

fn deserialize_slider<E: serde::de::Error>(obj: &serde_json::Map<String, Value>) -> Result<Element, E> {
    let slider = obj.get("slider")
        .and_then(|v| v.as_str())
        .ok_or_else(|| serde::de::Error::custom("slider field must be a string"))?
        .to_string();

    let id = obj.get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| serde::de::Error::custom("slider must have id field"))?
        .to_string();

    let min = obj.get("min")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| serde::de::Error::custom("slider must have min field (number)"))? as f32;

    let max = obj.get("max")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| serde::de::Error::custom("slider must have max field (number)"))? as f32;

    let default = obj.get("default").and_then(|v| v.as_f64()).map(|f| f as f32);

    let reveals = obj.get("reveals")
        .map(|v| serde_json::from_value::<Vec<Element>>(v.clone()))
        .transpose()
        .map_err(serde::de::Error::custom)?
        .unwrap_or_default();

    let when = obj.get("when").and_then(|v| v.as_str()).map(|s| s.to_string());

    Ok(Element::Slider { slider, id, min, max, default, reveals, when })
}

fn deserialize_checkbox<E: serde::de::Error>(obj: &serde_json::Map<String, Value>) -> Result<Element, E> {
    let checkbox = obj.get("checkbox")
        .and_then(|v| v.as_str())
        .ok_or_else(|| serde::de::Error::custom("checkbox field must be a string"))?
        .to_string();

    let id = obj.get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| serde::de::Error::custom("checkbox must have id field"))?
        .to_string();

    let default = obj.get("default").and_then(|v| v.as_bool()).unwrap_or(false);

    let reveals = obj.get("reveals")
        .map(|v| serde_json::from_value::<Vec<Element>>(v.clone()))
        .transpose()
        .map_err(serde::de::Error::custom)?
        .unwrap_or_default();

    let when = obj.get("when").and_then(|v| v.as_str()).map(|s| s.to_string());

    Ok(Element::Checkbox { checkbox, id, default, reveals, when })
}

fn deserialize_textbox<E: serde::de::Error>(obj: &serde_json::Map<String, Value>) -> Result<Element, E> {
    let textbox = obj.get("textbox")
        .and_then(|v| v.as_str())
        .ok_or_else(|| serde::de::Error::custom("textbox field must be a string"))?
        .to_string();

    let id = obj.get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| serde::de::Error::custom("textbox must have id field"))?
        .to_string();

    let placeholder = obj.get("placeholder").and_then(|v| v.as_str()).map(|s| s.to_string());
    let rows = obj.get("rows").and_then(|v| v.as_u64()).map(|n| n as u32);

    let reveals = obj.get("reveals")
        .map(|v| serde_json::from_value::<Vec<Element>>(v.clone()))
        .transpose()
        .map_err(serde::de::Error::custom)?
        .unwrap_or_default();

    let when = obj.get("when").and_then(|v| v.as_str()).map(|s| s.to_string());

    Ok(Element::Textbox { textbox, id, placeholder, rows, reveals, when })
}

fn deserialize_multiselect<E: serde::de::Error>(obj: &serde_json::Map<String, Value>) -> Result<Element, E> {
    let multiselect = obj.get("multiselect")
        .and_then(|v| v.as_str())
        .ok_or_else(|| serde::de::Error::custom("multiselect field must be a string"))?
        .to_string();

    let id = obj.get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| serde::de::Error::custom("multiselect must have id field"))?
        .to_string();

    let options = obj.get("options")
        .ok_or_else(|| serde::de::Error::custom("multiselect must have options field"))?;
    let options = serde_json::from_value::<Vec<String>>(options.clone())
        .map_err(serde::de::Error::custom)?;

    let reveals = obj.get("reveals")
        .map(|v| serde_json::from_value::<Vec<Element>>(v.clone()))
        .transpose()
        .map_err(serde::de::Error::custom)?
        .unwrap_or_default();

    let when = obj.get("when").and_then(|v| v.as_str()).map(|s| s.to_string());

    // Extract option-as-key children: any key that's not a known field and IS in options list
    let known_fields = ["multiselect", "id", "options", "reveals", "when"];
    let mut option_children = HashMap::new();

    for (key, value) in obj.iter() {
        if !known_fields.contains(&key.as_str()) && options.contains(key) {
            // This is an option-as-key mapping
            let children = serde_json::from_value::<Vec<Element>>(value.clone())
                .map_err(|e| serde::de::Error::custom(format!("Invalid children for option '{}': {}", key, e)))?;
            option_children.insert(key.clone(), children);
        }
    }

    Ok(Element::Multiselect { multiselect, id, options, option_children, reveals, when })
}

fn deserialize_choice<E: serde::de::Error>(obj: &serde_json::Map<String, Value>) -> Result<Element, E> {
    let choice = obj.get("choice")
        .and_then(|v| v.as_str())
        .ok_or_else(|| serde::de::Error::custom("choice field must be a string"))?
        .to_string();

    let id = obj.get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| serde::de::Error::custom("choice must have id field"))?
        .to_string();

    let options = obj.get("options")
        .ok_or_else(|| serde::de::Error::custom("choice must have options field"))?;
    let options = serde_json::from_value::<Vec<String>>(options.clone())
        .map_err(serde::de::Error::custom)?;

    let default = obj.get("default").and_then(|v| v.as_u64()).map(|n| n as usize);

    let reveals = obj.get("reveals")
        .map(|v| serde_json::from_value::<Vec<Element>>(v.clone()))
        .transpose()
        .map_err(serde::de::Error::custom)?
        .unwrap_or_default();

    let when = obj.get("when").and_then(|v| v.as_str()).map(|s| s.to_string());

    // Extract option-as-key children
    let known_fields = ["choice", "id", "options", "default", "reveals", "when"];
    let mut option_children = HashMap::new();

    for (key, value) in obj.iter() {
        if !known_fields.contains(&key.as_str()) && options.contains(key) {
            let children = serde_json::from_value::<Vec<Element>>(value.clone())
                .map_err(|e| serde::de::Error::custom(format!("Invalid children for option '{}': {}", key, e)))?;
            option_children.insert(key.clone(), children);
        }
    }

    Ok(Element::Choice { choice, id, options, default, option_children, reveals, when })
}

fn deserialize_group<E: serde::de::Error>(obj: &serde_json::Map<String, Value>) -> Result<Element, E> {
    let group = obj.get("group")
        .and_then(|v| v.as_str())
        .ok_or_else(|| serde::de::Error::custom("group field must be a string"))?
        .to_string();

    let id = obj.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());

    let elements = obj.get("elements")
        .ok_or_else(|| serde::de::Error::custom("group must have elements field"))?;
    let elements = serde_json::from_value::<Vec<Element>>(elements.clone())
        .map_err(serde::de::Error::custom)?;

    let when = obj.get("when").and_then(|v| v.as_str()).map(|s| s.to_string());

    Ok(Element::Group { group, id, elements, when })
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
            Element::Slider { slider, id, min, max, .. } => {
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
            "choice": "Theme",
            "id": "theme",
            "options": ["Light", "Dark"],
            "Dark": [
                {"text": "Dark mode settings"}
            ]
        }"#;
        let elem: Element = serde_json::from_str(json).unwrap();
        match elem {
            Element::Choice { choice, id, options, option_children, .. } => {
                assert_eq!(choice, "Theme");
                assert_eq!(id, "theme");
                assert_eq!(options, vec!["Light", "Dark"]);
                assert!(option_children.contains_key("Dark"));
                assert_eq!(option_children.get("Dark").unwrap().len(), 1);
            }
            _ => panic!("Expected Choice variant"),
        }
    }

    #[test]
    fn test_deserialize_multiselect_with_option_children() {
        let json = r#"{
            "multiselect": "Features",
            "id": "features",
            "options": ["Basic", "Advanced"],
            "Advanced": [
                {"slider": "Level", "id": "level", "min": 1, "max": 10}
            ]
        }"#;
        let elem: Element = serde_json::from_str(json).unwrap();
        match elem {
            Element::Multiselect { multiselect, id, options, option_children, .. } => {
                assert_eq!(multiselect, "Features");
                assert_eq!(id, "features");
                assert_eq!(options, vec!["Basic", "Advanced"]);
                assert!(option_children.contains_key("Advanced"));
                assert_eq!(option_children.get("Advanced").unwrap().len(), 1);
            }
            _ => panic!("Expected Multiselect variant"),
        }
    }

    #[test]
    fn test_serialize_slider_with_reveals() {
        let elem = Element::Slider {
            slider: "Volume".to_string(),
            id: "vol".to_string(),
            min: 0.0,
            max: 100.0,
            default: Some(75.0),
            reveals: vec![Element::Text {
                text: "High volume!".to_string(),
                id: None,
                when: Some("@vol > 80".to_string()),
            }],
            when: None,
        };
        let json = serde_json::to_value(&elem).unwrap();
        assert_eq!(json["slider"], "Volume");
        assert_eq!(json["id"], "vol");
        assert_eq!(json["default"], 75.0);
        assert!(json["reveals"].is_array());
        assert_eq!(json["reveals"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_roundtrip_choice_with_option_children() {
        let mut option_children = HashMap::new();
        option_children.insert("Dark".to_string(), vec![Element::Slider {
            slider: "Brightness".to_string(),
            id: "brightness".to_string(),
            min: 0.0,
            max: 100.0,
            default: Some(50.0),
            reveals: vec![],
            when: None,
        }]);

        let original = Element::Choice {
            choice: "Theme".to_string(),
            id: "theme".to_string(),
            options: vec!["Light".to_string(), "Dark".to_string()],
            default: Some(1),
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
        option_children.insert("Advanced".to_string(), vec![Element::Text {
            text: "Advanced mode".to_string(),
            id: None,
            when: None,
        }]);

        let elem = Element::Choice {
            choice: "Mode".to_string(),
            id: "mode".to_string(),
            options: vec!["Basic".to_string(), "Advanced".to_string()],
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
            Element::Slider {
                slider: "Volume".to_string(),
                id: "vol".to_string(),
                min: 0.0,
                max: 100.0,
                default: None,
                reveals: vec![],
                when: None,
            },
            Element::Checkbox {
                checkbox: "Enable".to_string(),
                id: "enabled".to_string(),
                default: true,
                reveals: vec![],
                when: None,
            },
            Element::Textbox {
                textbox: "Name".to_string(),
                id: "name".to_string(),
                placeholder: Some("Enter name".to_string()),
                rows: Some(3),
                reveals: vec![],
                when: None,
            },
            Element::Multiselect {
                multiselect: "Options".to_string(),
                id: "opts".to_string(),
                options: vec!["A".to_string(), "B".to_string()],
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
            assert_eq!(deserialized, original, "Round-trip failed for {:?}", original);
        }
    }
}
