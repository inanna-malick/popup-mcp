use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PopupDefinition {
    pub title: String,
    pub elements: Vec<Element>,
}

#[derive(Debug, Clone)]
pub enum Element {
    Text(String),
    Slider {
        label: String,
        min: f32,
        max: f32,
        default: f32,
    },
    Checkbox {
        label: String,
        default: bool,
    },
    Textbox {
        label: String,
        placeholder: Option<String>,
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
    Buttons(Vec<String>),
    Conditional {
        condition: Condition,
        elements: Vec<Element>,
    },
}

#[derive(Debug, Clone)]
pub enum Condition {
    Checked(String),
    Selected(String, String),
    Count(String, ComparisonOp, i32),
}

#[derive(Debug, Clone)]
pub enum ComparisonOp {
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    Equal,
}

/// Runtime state of the popup
#[derive(Default)]
pub struct PopupState {
    pub sliders: HashMap<String, f32>,
    pub checkboxes: HashMap<String, bool>,
    pub textboxes: HashMap<String, String>,
    pub choices: HashMap<String, usize>,
    pub multiselects: HashMap<String, Vec<bool>>,
    pub button_clicked: Option<String>,
}

impl PopupState {
    pub fn new(definition: &PopupDefinition) -> Self {
        let mut state = PopupState::default();
        
        // Initialize with defaults from definition
        state.init_elements(&definition.elements);
        state
    }
    
    fn init_elements(&mut self, elements: &[Element]) {
        for element in elements {
            match element {
                Element::Slider { label, default, .. } => {
                    self.sliders.insert(label.clone(), *default);
                }
                Element::Checkbox { label, default } => {
                    self.checkboxes.insert(label.clone(), *default);
                }
                Element::Textbox { label, .. } => {
                    self.textboxes.insert(label.clone(), String::new());
                }
                Element::Choice { label, .. } => {
                    self.choices.insert(label.clone(), 0); // First option selected by default
                }
                Element::Multiselect { label, options } => {
                    self.multiselects.insert(label.clone(), vec![false; options.len()]);
                }
                Element::Group { elements, .. } => {
                    self.init_elements(elements);
                }
                Element::Conditional { elements, .. } => {
                    // Initialize nested elements too
                    self.init_elements(elements);
                }
                _ => {}
            }
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
        let mut values = HashMap::new();
        
        // Add all values from state
        for (key, value) in &state.sliders {
            values.insert(key.clone(), json!(*value as i32));
        }
        
        for (key, value) in &state.checkboxes {
            values.insert(key.clone(), json!(*value));
        }
        
        for (key, value) in &state.textboxes {
            if !value.is_empty() {
                values.insert(key.clone(), json!(value));
            }
        }
        
        for (key, value) in &state.choices {
            values.insert(key.clone(), json!(*value));
        }
        
        for (key, selections) in &state.multiselects {
            // Convert Vec<bool> to indices of selected items
            let selected_indices: Vec<usize> = selections
                .iter()
                .enumerate()
                .filter_map(|(i, &selected)| if selected { Some(i) } else { None })
                .collect();
            values.insert(key.clone(), json!(selected_indices));
        }
        
        PopupResult {
            values,
            button: state.button_clicked.clone().unwrap_or_else(|| "cancel".to_string()),
        }
    }
}
