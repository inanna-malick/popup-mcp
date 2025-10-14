pub mod protocol;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Unique identifier for widget state values, combining element path and label.
/// Prevents state collisions when same label appears in multiple conditional branches.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StateKey {
    /// Full path including element position (e.g., "0.multiselect_1.Why?")
    full_path: String,
    /// Original widget label for result serialization (e.g., "Why?")
    label: String,
}

impl StateKey {
    /// Create a new state key from label and element path
    pub fn new(label: impl Into<String>, element_path: &str) -> Self {
        let label = label.into();
        let full_path = if element_path.is_empty() {
            label.clone()
        } else {
            format!("{}.{}", element_path, label)
        };
        Self { full_path, label }
    }

    /// Get the original label (for result serialization)
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Get the full path (for debugging)
    pub fn full_path(&self) -> &str {
        &self.full_path
    }
}

/// Represents an option value in Choice or Multiselect elements.
/// Can be a simple string or include inline conditional elements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OptionValue {
    /// Simple string option with no conditional UI
    Simple(String),
    /// Option with conditional elements shown when selected
    WithConditional {
        value: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        conditional: Vec<Element>,
    },
}

impl OptionValue {
    /// Get the display value of this option
    pub fn value(&self) -> &str {
        match self {
            OptionValue::Simple(s) => s,
            OptionValue::WithConditional { value, .. } => value,
        }
    }

    /// Get the conditional elements if present
    pub fn conditional(&self) -> Option<&[Element]> {
        match self {
            OptionValue::Simple(_) => None,
            OptionValue::WithConditional { conditional, .. } => {
                if conditional.is_empty() {
                    None
                } else {
                    Some(conditional)
                }
            }
        }
    }
}

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
        /// Optional inline conditional elements shown when checkbox is checked
        #[serde(default, skip_serializing_if = "Option::is_none")]
        conditional: Option<Vec<Element>>,
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
        options: Vec<OptionValue>,
    },
    Choice {
        label: String,
        options: Vec<OptionValue>,
        #[serde(default)]
        default: Option<usize>,
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
    Choice(Option<usize>),
}

/// Runtime state of the popup
#[derive(Default)]
pub struct PopupState {
    pub values: HashMap<StateKey, ElementValue>,
    pub button_clicked: Option<String>,
}

impl PopupState {
    pub fn new(definition: &PopupDefinition) -> Self {
        let mut state = PopupState::default();
        state.init_elements(&definition.elements, "");
        state
    }

    fn init_elements(&mut self, elements: &[Element], path_prefix: &str) {
        for (idx, element) in elements.iter().enumerate() {
            let element_path = if path_prefix.is_empty() {
                idx.to_string()
            } else {
                format!("{}.{}", path_prefix, idx)
            };
            match element {
                Element::Slider {
                    label,
                    min,
                    max,
                    default,
                } => {
                    let default_value = default.unwrap_or((min + max) / 2.0);
                    self.values.insert(
                        StateKey::new(label, &element_path),
                        ElementValue::Number(default_value),
                    );
                }
                Element::Checkbox {
                    label,
                    default,
                    conditional,
                } => {
                    self.values.insert(
                        StateKey::new(label, &element_path),
                        ElementValue::Boolean(*default),
                    );
                    // Recursively init inline conditional elements
                    if let Some(children) = conditional {
                        self.init_elements(children, &format!("{}.checkbox", element_path));
                    }
                }
                Element::Textbox { label, .. } => {
                    self.values.insert(
                        StateKey::new(label, &element_path),
                        ElementValue::Text(String::new()),
                    );
                }
                Element::Multiselect { label, options } => {
                    self.values.insert(
                        StateKey::new(label, &element_path),
                        ElementValue::MultiChoice(vec![false; options.len()]),
                    );
                    // Recursively init inline conditionals from options
                    for (i, opt) in options.iter().enumerate() {
                        if let Some(children) = opt.conditional() {
                            self.init_elements(
                                children,
                                &format!("{}.multiselect_{}", element_path, i),
                            );
                        }
                    }
                }
                Element::Choice {
                    label,
                    default,
                    options,
                } => {
                    self.values.insert(
                        StateKey::new(label, &element_path),
                        ElementValue::Choice(*default),
                    );
                    // Recursively init inline conditionals from options
                    for (i, opt) in options.iter().enumerate() {
                        if let Some(children) = opt.conditional() {
                            self.init_elements(children, &format!("{}.choice_{}", element_path, i));
                        }
                    }
                }
                Element::Group { elements, .. } => {
                    self.init_elements(elements, &format!("{}.group", element_path));
                }
                Element::Conditional { elements, .. } => {
                    self.init_elements(elements, &format!("{}.cond", element_path));
                }
                _ => {}
            }
        }
    }

    // Helper methods for GUI access
    pub fn get_number_mut(&mut self, key: &StateKey) -> Option<&mut f32> {
        match self.values.get_mut(key) {
            Some(ElementValue::Number(ref mut n)) => Some(n),
            _ => None,
        }
    }

    pub fn get_boolean_mut(&mut self, key: &StateKey) -> Option<&mut bool> {
        match self.values.get_mut(key) {
            Some(ElementValue::Boolean(ref mut b)) => Some(b),
            _ => None,
        }
    }

    pub fn get_text_mut(&mut self, key: &StateKey) -> Option<&mut String> {
        match self.values.get_mut(key) {
            Some(ElementValue::Text(ref mut s)) => Some(s),
            _ => None,
        }
    }

    pub fn get_multichoice_mut(&mut self, key: &StateKey) -> Option<&mut Vec<bool>> {
        match self.values.get_mut(key) {
            Some(ElementValue::MultiChoice(ref mut v)) => Some(v),
            _ => None,
        }
    }

    pub fn get_choice_mut(&mut self, key: &StateKey) -> Option<&mut Option<usize>> {
        match self.values.get_mut(key) {
            Some(ElementValue::Choice(ref mut c)) => Some(c),
            _ => None,
        }
    }

    pub fn get_boolean(&self, key: &StateKey) -> bool {
        match self.values.get(key) {
            Some(ElementValue::Boolean(b)) => *b,
            _ => false,
        }
    }

    pub fn get_multichoice(&self, key: &StateKey) -> Option<&Vec<bool>> {
        match self.values.get(key) {
            Some(ElementValue::MultiChoice(v)) => Some(v),
            _ => None,
        }
    }

    pub fn get_text(&self, key: &StateKey) -> Option<&String> {
        match self.values.get(key) {
            Some(ElementValue::Text(s)) => Some(s),
            _ => None,
        }
    }

    pub fn get_choice(&self, key: &StateKey) -> Option<Option<usize>> {
        match self.values.get(key) {
            Some(ElementValue::Choice(c)) => Some(*c),
            _ => None,
        }
    }

    /// Find StateKey by label - useful for tests and simple lookups
    /// Returns first matching key if multiple elements share same label
    pub fn find_key_by_label(&self, label: &str) -> Option<StateKey> {
        self.values.keys().find(|k| k.label() == label).cloned()
    }
}

/// Result that gets serialized to JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum PopupResult {
    #[serde(rename = "completed")]
    Completed {
        #[serde(flatten)]
        values: HashMap<String, Value>,
        button: String,
    },
    #[serde(rename = "cancelled")]
    Cancelled,
    #[serde(rename = "timeout")]
    Timeout { message: String },
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
                Some((key.label().to_string(), json_value))
            })
            .collect();

        PopupResult::Completed {
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
                    | e @ Element::Choice { label: l, .. }
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
            let element = find_element(&definition.elements, key.label());

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
                            selected
                                .then_some(options.get(i).map(|opt| opt.value().to_string()))
                                .flatten()
                        })
                        .collect();
                    json!(selected)
                }
                (ElementValue::Choice(Some(idx)), Some(Element::Choice { options, .. })) => {
                    // Return selected option text
                    options
                        .get(*idx)
                        .map(|opt| json!(opt.value()))
                        .unwrap_or_else(|| json!(null))
                }
                (ElementValue::Choice(None), _) => continue, // Skip unselected choice
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

            values.insert(key.label().to_string(), json_value);
        }

        PopupResult::Completed {
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
                    | e @ Element::Choice { label: l, .. }
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
            if !active_labels.contains(&key.label().to_string()) {
                continue;
            }

            let element = find_element(&definition.elements, key.label());

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
                            selected
                                .then_some(options.get(i).map(|opt| opt.value().to_string()))
                                .flatten()
                        })
                        .collect();
                    json!(selected)
                }
                (ElementValue::Choice(Some(idx)), Some(Element::Choice { options, .. })) => {
                    // Return selected option text
                    options
                        .get(*idx)
                        .map(|opt| json!(opt.value()))
                        .unwrap_or_else(|| json!(null))
                }
                (ElementValue::Choice(None), _) => continue, // Skip unselected choice
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

            values.insert(key.label().to_string(), json_value);
        }

        PopupResult::Completed {
            values,
            button: state
                .button_clicked
                .clone()
                .unwrap_or_else(|| "cancel".to_string()),
        }
    }
}
