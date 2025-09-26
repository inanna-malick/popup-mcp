use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PopupDefinition {
    pub title: Option<String>,
    pub elements: Vec<Element>,
}

impl PopupDefinition {
    /// Get the effective title, falling back to a default if none provided
    pub fn effective_title(&self) -> &str {
        self.title.as_deref().unwrap_or("Dialog")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Element {
    Text {
        content: String,
    },
    Slider {
        label: String,
        min: f32,
        max: f32,
        #[serde(default)]
        default: Option<f32>,
    },
    Checkbox {
        label: String,
        #[serde(default)]
        default: bool,
    },
    Textbox {
        label: String,
        #[serde(default)]
        placeholder: Option<String>,
        #[serde(default)]
        rows: Option<u32>,
    },
    Multiselect {
        label: String,
        options: Vec<String>,
    },
    Group {
        label: String,
        elements: Vec<Element>,
    },
    Conditional {
        condition: Condition,
        elements: Vec<Element>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Condition {
    // Pattern 1: Simple existence check - true if checkbox checked OR any multiselect option selected
    Simple(String),
    // Pattern 2: Specific value check - true if checkbox name matches value OR multiselect has option selected
    Field {
        field: String,
        value: String,
    },
    // Pattern 3: Quantity check - count of selected items (checkbox: 0 or 1, multiselect: count of selected)
    Count {
        field: String,
        count: String, // e.g., ">2", "=1", "<=3"
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonOp {
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    Equal,
}

impl ComparisonOp {
    /// Parse a count condition string like ">2", "=1", "<=3" into operator and value
    pub fn parse_count_condition(count_str: &str) -> Option<(ComparisonOp, i32)> {
        let count_str = count_str.trim();

        if let Some(rest) = count_str.strip_prefix(">=") {
            rest.parse().ok().map(|v| (ComparisonOp::GreaterEqual, v))
        } else if let Some(rest) = count_str.strip_prefix("<=") {
            rest.parse().ok().map(|v| (ComparisonOp::LessEqual, v))
        } else if let Some(rest) = count_str.strip_prefix(">") {
            rest.parse().ok().map(|v| (ComparisonOp::Greater, v))
        } else if let Some(rest) = count_str.strip_prefix("<") {
            rest.parse().ok().map(|v| (ComparisonOp::Less, v))
        } else if let Some(rest) = count_str.strip_prefix("=") {
            rest.parse().ok().map(|v| (ComparisonOp::Equal, v))
        } else {
            // Default to equality if no operator specified
            count_str.parse().ok().map(|v| (ComparisonOp::Equal, v))
        }
    }
}

// Custom serialization/deserialization to handle operator strings
impl Serialize for ComparisonOp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            ComparisonOp::Greater => ">",
            ComparisonOp::Less => "<",
            ComparisonOp::GreaterEqual => ">=",
            ComparisonOp::LessEqual => "<=",
            ComparisonOp::Equal => "=",
        };
        serializer.serialize_str(s)
    }
}

impl<'de> Deserialize<'de> for ComparisonOp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            ">" => Ok(ComparisonOp::Greater),
            "<" => Ok(ComparisonOp::Less),
            ">=" => Ok(ComparisonOp::GreaterEqual),
            "<=" => Ok(ComparisonOp::LessEqual),
            "=" | "==" => Ok(ComparisonOp::Equal),
            _ => Err(serde::de::Error::custom(format!(
                "Invalid comparison operator: {}. Expected one of: >, <, >=, <=, =",
                s
            ))),
        }
    }
}

/// Unified value type for all widget states
#[derive(Debug, Clone)]
pub enum ElementValue {
    Number(f32),
    Boolean(bool),
    Text(String),
    MultiChoice(Vec<bool>),
}

/// Runtime state of the popup
#[derive(Default)]
pub struct PopupState {
    pub values: HashMap<String, ElementValue>,
    pub button_clicked: Option<String>,
}

impl PopupState {
    pub fn new(definition: &PopupDefinition) -> Self {
        let mut state = PopupState::default();
        state.init_elements(&definition.elements);
        state
    }

    fn init_elements(&mut self, elements: &[Element]) {
        for element in elements {
            match element {
                Element::Slider {
                    label,
                    min,
                    max,
                    default,
                } => {
                    let default_value = default.unwrap_or((min + max) / 2.0);
                    self.values
                        .insert(label.clone(), ElementValue::Number(default_value));
                }
                Element::Checkbox { label, default } => {
                    self.values
                        .insert(label.clone(), ElementValue::Boolean(*default));
                }
                Element::Textbox { label, .. } => {
                    self.values
                        .insert(label.clone(), ElementValue::Text(String::new()));
                }
                Element::Multiselect { label, options } => {
                    self.values.insert(
                        label.clone(),
                        ElementValue::MultiChoice(vec![false; options.len()]),
                    );
                }
                Element::Group { elements, .. } | Element::Conditional { elements, .. } => {
                    self.init_elements(elements);
                }
                _ => {}
            }
        }
    }

    // Helper methods for GUI access
    pub fn get_number_mut(&mut self, label: &str) -> Option<&mut f32> {
        match self.values.get_mut(label) {
            Some(ElementValue::Number(ref mut n)) => Some(n),
            _ => None,
        }
    }

    pub fn get_boolean_mut(&mut self, label: &str) -> Option<&mut bool> {
        match self.values.get_mut(label) {
            Some(ElementValue::Boolean(ref mut b)) => Some(b),
            _ => None,
        }
    }

    pub fn get_text_mut(&mut self, label: &str) -> Option<&mut String> {
        match self.values.get_mut(label) {
            Some(ElementValue::Text(ref mut s)) => Some(s),
            _ => None,
        }
    }


    pub fn get_multichoice_mut(&mut self, label: &str) -> Option<&mut Vec<bool>> {
        match self.values.get_mut(label) {
            Some(ElementValue::MultiChoice(ref mut v)) => Some(v),
            _ => None,
        }
    }

    pub fn get_boolean(&self, label: &str) -> bool {
        match self.values.get(label) {
            Some(ElementValue::Boolean(b)) => *b,
            _ => false,
        }
    }


    pub fn get_multichoice(&self, label: &str) -> Option<&Vec<bool>> {
        match self.values.get(label) {
            Some(ElementValue::MultiChoice(v)) => Some(v),
            _ => None,
        }
    }

    pub fn get_text(&self, label: &str) -> Option<&String> {
        match self.values.get(label) {
            Some(ElementValue::Text(s)) => Some(s),
            _ => None,
        }
    }
}

/// Result that gets serialized to JSON
#[derive(Debug, Serialize, Deserialize)]
pub struct PopupResult {
    #[serde(flatten)]
    pub values: HashMap<String, Value>,
    pub button: String,
}

impl PopupResult {
    pub fn from_state(state: &PopupState) -> Self {
        let values = state
            .values
            .iter()
            .filter_map(|(key, value)| {
                let json_value = match value {
                    ElementValue::Number(n) => json!(*n as i32),
                    ElementValue::Boolean(b) => json!(*b),
                    ElementValue::Text(s) if !s.is_empty() => json!(s),
                    ElementValue::MultiChoice(selections) => {
                        let indices: Vec<usize> = selections
                            .iter()
                            .enumerate()
                            .filter_map(|(i, &selected)| selected.then_some(i))
                            .collect();
                        json!(indices)
                    }
                    _ => return None, // Skip empty text fields
                };
                Some((key.clone(), json_value))
            })
            .collect();

        PopupResult {
            values,
            button: state
                .button_clicked
                .clone()
                .unwrap_or_else(|| "cancel".to_string()),
        }
    }

    pub fn from_state_with_context(state: &PopupState, definition: &PopupDefinition) -> Self {
        let mut values = HashMap::new();

        // Helper to find element metadata by label
        fn find_element<'a>(elements: &'a [Element], label: &str) -> Option<&'a Element> {
            for element in elements {
                match element {
                    e @ Element::Slider { label: l, .. }
                    | e @ Element::Checkbox { label: l, .. }
                    | e @ Element::Textbox { label: l, .. }
                    | e @ Element::Multiselect { label: l, .. }
                        if l == label =>
                    {
                        return Some(e)
                    }
                    Element::Group {
                        elements: group_elements,
                        ..
                    } => {
                        if let Some(e) = find_element(group_elements, label) {
                            return Some(e);
                        }
                    }
                    Element::Conditional {
                        elements: cond_elements,
                        ..
                    } => {
                        if let Some(e) = find_element(cond_elements, label) {
                            return Some(e);
                        }
                    }
                    _ => {}
                }
            }
            None
        }

        for (key, value) in &state.values {
            let element = find_element(&definition.elements, key);

            let json_value = match (value, element) {
                (ElementValue::Number(n), Some(Element::Slider { max, .. })) => {
                    // Format as "value/max" for sliders
                    json!(format!("{}/{}", *n as i32, *max as i32))
                }
                (ElementValue::Boolean(b), _) => json!(*b),
                (ElementValue::Text(s), _) if !s.is_empty() => json!(s),
                (
                    ElementValue::MultiChoice(selections),
                    Some(Element::Multiselect { options, .. }),
                ) => {
                    // Return selected option texts instead of indices
                    let selected: Vec<String> = selections
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &selected)| {
                            selected.then_some(options.get(i).cloned()).flatten()
                        })
                        .collect();
                    json!(selected)
                }
                (ElementValue::Number(n), _) => json!(*n as i32),
                (ElementValue::MultiChoice(selections), _) => {
                    let indices: Vec<usize> = selections
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &selected)| selected.then_some(i))
                        .collect();
                    json!(indices)
                }
                _ => continue, // Skip empty text fields
            };

            values.insert(key.clone(), json_value);
        }

        PopupResult {
            values,
            button: state
                .button_clicked
                .clone()
                .unwrap_or_else(|| "cancel".to_string()),
        }
    }

    pub fn from_state_with_active_elements(
        state: &PopupState,
        definition: &PopupDefinition,
        active_labels: &[String],
    ) -> Self {
        let mut values = HashMap::new();

        // Helper to find element metadata by label
        fn find_element<'a>(elements: &'a [Element], label: &str) -> Option<&'a Element> {
            for element in elements {
                match element {
                    e @ Element::Slider { label: l, .. }
                    | e @ Element::Checkbox { label: l, .. }
                    | e @ Element::Textbox { label: l, .. }
                    | e @ Element::Multiselect { label: l, .. }
                        if l == label =>
                    {
                        return Some(e)
                    }
                    Element::Group {
                        elements: group_elements,
                        ..
                    } => {
                        if let Some(e) = find_element(group_elements, label) {
                            return Some(e);
                        }
                    }
                    Element::Conditional {
                        elements: cond_elements,
                        ..
                    } => {
                        if let Some(e) = find_element(cond_elements, label) {
                            return Some(e);
                        }
                    }
                    _ => {}
                }
            }
            None
        }

        for (key, value) in &state.values {
            // Skip this value if it's not in the active elements
            if !active_labels.contains(key) {
                continue;
            }

            let element = find_element(&definition.elements, key);

            let json_value = match (value, element) {
                (ElementValue::Number(n), Some(Element::Slider { max, .. })) => {
                    // Format as "value/max" for sliders
                    json!(format!("{}/{}", *n as i32, *max as i32))
                }
                (ElementValue::Boolean(b), _) => json!(*b),
                (ElementValue::Text(s), _) if !s.is_empty() => json!(s),
                (
                    ElementValue::MultiChoice(selections),
                    Some(Element::Multiselect { options, .. }),
                ) => {
                    // Return selected option texts instead of indices
                    let selected: Vec<String> = selections
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &selected)| {
                            selected.then_some(options.get(i).cloned()).flatten()
                        })
                        .collect();
                    json!(selected)
                }
                (ElementValue::Number(n), _) => json!(*n as i32),
                (ElementValue::MultiChoice(selections), _) => {
                    let indices: Vec<usize> = selections
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &selected)| selected.then_some(i))
                        .collect();
                    json!(indices)
                }
                _ => continue, // Skip empty text fields
            };

            values.insert(key.clone(), json_value);
        }

        PopupResult {
            values,
            button: state
                .button_clicked
                .clone()
                .unwrap_or_else(|| "cancel".to_string()),
        }
    }
}
