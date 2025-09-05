use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PopupDefinition {
    pub title: String,
    pub elements: Vec<Element>,
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
    Choice {
        label: String,
        options: Vec<String>,
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
    // Simple string format: just the checkbox name
    Simple(String),
    // Complex conditions as objects
    Checked {
        checked: String,
    },
    Selected {
        selected: String,
        value: String,
    },
    Count {
        count: String,
        op: ComparisonOp,
        value: i32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComparisonOp {
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    Equal,
}

/// Unified value type for all widget states
#[derive(Debug, Clone)]
pub enum ElementValue {
    Number(f32),
    Boolean(bool),
    Text(String),
    Choice(usize),
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
                Element::Choice { label, .. } => {
                    self.values.insert(label.clone(), ElementValue::Choice(0));
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

    pub fn get_choice_mut(&mut self, label: &str) -> Option<&mut usize> {
        match self.values.get_mut(label) {
            Some(ElementValue::Choice(ref mut i)) => Some(i),
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

    pub fn get_choice(&self, label: &str) -> usize {
        match self.values.get(label) {
            Some(ElementValue::Choice(i)) => *i,
            _ => 0,
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
                    ElementValue::Choice(i) => json!(*i),
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
                    | e @ Element::Choice { label: l, .. }
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
                (ElementValue::Choice(i), Some(Element::Choice { options, .. })) => {
                    // Return the actual option text instead of index
                    json!(options.get(*i).cloned().unwrap_or_else(|| i.to_string()))
                }
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
                (ElementValue::Choice(i), _) => json!(*i),
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
