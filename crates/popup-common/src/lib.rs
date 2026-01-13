pub mod condition;
mod element_deser;

#[cfg(test)]
mod tests;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub use condition::{evaluate_condition, parse_condition, ConditionExpr};

/// Option value for Choice/Multiselect - can be simple string or with description
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum OptionValue {
    /// Simple string option
    Simple(String),
    /// Option with description (supports both "description" and "because" as aliases)
    WithDescription {
        value: String,
        #[serde(alias = "because")]
        description: String,
    },
}

impl OptionValue {
    /// Get the option value string
    pub fn value(&self) -> &str {
        match self {
            OptionValue::Simple(s) => s,
            OptionValue::WithDescription { value, .. } => value,
        }
    }

    /// Get the description if present
    pub fn description(&self) -> Option<&str> {
        match self {
            OptionValue::Simple(_) => None,
            OptionValue::WithDescription { description, .. } => Some(description),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PopupDefinition {
    pub title: String,
    pub elements: Vec<Element>,
}

impl PopupDefinition {
    /// Get the effective title
    pub fn effective_title(&self) -> &str {
        &self.title
    }
}

/// Schema v2: Element types using element-as-key pattern
/// Discriminated by which field is present, not explicit "type" tag
/// Serialize/Deserialize impls in element_deser.rs for element-as-key and option-as-key support
#[derive(Debug, Clone, PartialEq, JsonSchema)]
pub enum Element {
    /// Static text display
    Text {
        text: String,
        id: Option<String>,
        when: Option<String>,
    },

    /// Rich markdown text display
    Markdown {
        markdown: String, // Markdown content becomes the discriminator key
        id: Option<String>,
        when: Option<String>,
    },

    /// Numeric slider input
    Slider {
        slider: String, // Label text becomes the discriminator key
        id: String,
        min: f32,
        max: f32,
        default: Option<f32>,
        when: Option<String>,
    },

    /// Boolean checkbox input
    Check {
        check: String, // Label text becomes the discriminator key
        id: String,
        default: bool,
        reveals: Vec<Element>,
        when: Option<String>,
    },

    /// Text input field (single or multi-line)
    Input {
        input: String, // Label text becomes the discriminator key
        id: String,
        placeholder: Option<String>,
        rows: Option<u32>,
        when: Option<String>,
    },

    /// Multiple selection from options (with option-as-key nesting)
    Multi {
        multi: String, // Label text becomes the discriminator key
        id: String,
        options: Vec<OptionValue>,
        // Option-as-key nesting: HashMap<option_value, Vec<Element>>
        // Custom serialize/deserialize handles option children as direct JSON keys
        option_children: HashMap<String, Vec<Element>>,
        reveals: Vec<Element>,
        when: Option<String>,
    },

    /// Single selection from options (with option-as-key nesting)
    Select {
        select: String, // Label text becomes the discriminator key
        id: String,
        options: Vec<OptionValue>,
        default: Option<String>, // Default value (option text), not index
        // Option-as-key nesting: HashMap<option_value, Vec<Element>>
        // Custom serialize/deserialize handles option children as direct JSON keys
        option_children: HashMap<String, Vec<Element>>,
        reveals: Vec<Element>,
        when: Option<String>,
    },

    /// Labeled container for grouping elements
    Group {
        group: String, // Label text becomes the discriminator key
        id: Option<String>,
        elements: Vec<Element>,
        when: Option<String>,
    },
}

/// Unified value type for all widget states
#[derive(Debug, Clone, PartialEq, JsonSchema)]
pub enum ElementValue {
    Number(f32),
    Boolean(bool),
    Text(String),
    MultiChoice(Vec<bool>),
    Choice(Option<usize>),
}

/// Runtime state of the popup (v2 schema)
#[derive(Default)]
pub struct PopupState {
    pub values: HashMap<String, ElementValue>, // ID -> value
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
                    id,
                    min,
                    max,
                    default,
                    ..
                } => {
                    let default_value = default.unwrap_or((min + max) / 2.0);
                    self.values
                        .insert(id.clone(), ElementValue::Number(default_value));
                }
                Element::Check {
                    id,
                    default,
                    reveals,
                    ..
                } => {
                    self.values
                        .insert(id.clone(), ElementValue::Boolean(*default));
                    self.init_elements(reveals);
                }
                Element::Input { id, .. } => {
                    self.values
                        .insert(id.clone(), ElementValue::Text(String::new()));
                }
                Element::Multi {
                    id,
                    options,
                    option_children,
                    reveals,
                    ..
                } => {
                    self.values.insert(
                        id.clone(),
                        ElementValue::MultiChoice(vec![false; options.len()]),
                    ); // options.len() still works with Vec<OptionValue>
                       // Recurse into option-as-key children
                    for children in option_children.values() {
                        self.init_elements(children);
                    }
                    self.init_elements(reveals);
                }
                Element::Select {
                    id,
                    options,
                    default,
                    option_children,
                    reveals,
                    ..
                } => {
                    // Convert string default to index
                    let default_idx = default.as_ref().and_then(|default_val| {
                        options.iter().position(|opt| opt.value() == default_val)
                    });
                    self.values
                        .insert(id.clone(), ElementValue::Choice(default_idx));
                    for children in option_children.values() {
                        self.init_elements(children);
                    }
                    self.init_elements(reveals);
                }
                Element::Group { elements, .. } => {
                    self.init_elements(elements);
                }
                Element::Text { .. } => {
                    // Text elements have no state
                }
                Element::Markdown { .. } => {
                    // Markdown elements have no state
                }
            }
        }
    }

    // Helper methods for GUI access - now take &str (id) instead of &StateKey
    pub fn get_number_mut(&mut self, id: &str) -> Option<&mut f32> {
        match self.values.get_mut(id) {
            Some(ElementValue::Number(ref mut n)) => Some(n),
            _ => None,
        }
    }

    pub fn get_boolean_mut(&mut self, id: &str) -> Option<&mut bool> {
        match self.values.get_mut(id) {
            Some(ElementValue::Boolean(ref mut b)) => Some(b),
            _ => None,
        }
    }

    pub fn get_text_mut(&mut self, id: &str) -> Option<&mut String> {
        match self.values.get_mut(id) {
            Some(ElementValue::Text(ref mut s)) => Some(s),
            _ => None,
        }
    }

    pub fn get_multichoice_mut(&mut self, id: &str) -> Option<&mut Vec<bool>> {
        match self.values.get_mut(id) {
            Some(ElementValue::MultiChoice(ref mut v)) => Some(v),
            _ => None,
        }
    }

    pub fn get_choice_mut(&mut self, id: &str) -> Option<&mut Option<usize>> {
        match self.values.get_mut(id) {
            Some(ElementValue::Choice(ref mut c)) => Some(c),
            _ => None,
        }
    }

    // Const accessors for condition evaluation
    pub fn get_boolean(&self, id: &str) -> bool {
        match self.values.get(id) {
            Some(ElementValue::Boolean(b)) => *b,
            _ => false,
        }
    }

    pub fn get_multichoice(&self, id: &str) -> Option<&Vec<bool>> {
        match self.values.get(id) {
            Some(ElementValue::MultiChoice(v)) => Some(v),
            _ => None,
        }
    }

    pub fn get_choice(&self, id: &str) -> Option<Option<usize>> {
        match self.values.get(id) {
            Some(ElementValue::Choice(c)) => Some(*c),
            _ => None,
        }
    }

    pub fn get_text(&self, id: &str) -> Option<&String> {
        match self.values.get(id) {
            Some(ElementValue::Text(s)) => Some(s),
            _ => None,
        }
    }

    /// Convert PopupState to HashMap<String, Value> for condition evaluation
    /// Maps id -> JSON value for use with evaluate_condition()
    ///
    /// For Choice fields: converts index to selected option text (or null if none selected)
    /// For Multiselect fields: converts to array of selected option texts
    pub fn to_value_map(&self, elements: &[Element]) -> HashMap<String, Value> {
        use serde_json::json;

        // Helper to find element by ID
        fn find_element_by_id<'a>(elements: &'a [Element], target_id: &str) -> Option<&'a Element> {
            for element in elements {
                match element {
                    Element::Text { id: Some(id), .. } if id == target_id => return Some(element),
                    Element::Slider { id, .. } if id == target_id => return Some(element),
                    Element::Check { id, .. } if id == target_id => return Some(element),
                    Element::Input { id, .. } if id == target_id => return Some(element),
                    Element::Multi { id, .. } if id == target_id => return Some(element),
                    Element::Select { id, .. } if id == target_id => return Some(element),
                    Element::Group { id: Some(id), .. } if id == target_id => return Some(element),

                    // Recurse into nested structures
                    Element::Group {
                        elements: nested, ..
                    } => {
                        if let Some(e) = find_element_by_id(nested, target_id) {
                            return Some(e);
                        }
                    }
                    Element::Check { reveals, .. } => {
                        if let Some(e) = find_element_by_id(reveals, target_id) {
                            return Some(e);
                        }
                    }
                    Element::Slider { .. } | Element::Input { .. } => {
                        // Slider and Input have no reveals, nothing to search
                    }
                    Element::Multi {
                        reveals,
                        option_children,
                        ..
                    }
                    | Element::Select {
                        reveals,
                        option_children,
                        ..
                    } => {
                        if let Some(e) = find_element_by_id(reveals, target_id) {
                            return Some(e);
                        }
                        for children in option_children.values() {
                            if let Some(e) = find_element_by_id(children, target_id) {
                                return Some(e);
                            }
                        }
                    }
                    _ => {}
                }
            }
            None
        }

        self.values
            .iter()
            .map(|(id, val)| {
                let json_val = match val {
                    ElementValue::Number(n) => json!(*n),
                    ElementValue::Boolean(b) => json!(*b),
                    ElementValue::Text(s) => json!(s),
                    ElementValue::MultiChoice(selections) => {
                        // Convert to array of selected option texts for selected() function
                        if let Some(Element::Multi { options, .. }) =
                            find_element_by_id(elements, id)
                        {
                            let selected_texts: Vec<&str> = selections
                                .iter()
                                .enumerate()
                                .filter_map(|(i, &selected)| {
                                    if selected {
                                        options.get(i).map(|opt| opt.value())
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            json!(selected_texts)
                        } else {
                            // Fallback: array of booleans
                            json!(selections)
                        }
                    }
                    ElementValue::Choice(idx) => {
                        // Convert index to option text for selected() function
                        if let Some(Element::Select { options, .. }) =
                            find_element_by_id(elements, id)
                        {
                            match idx {
                                Some(i) => options
                                    .get(*i)
                                    .map(|opt| json!(opt.value()))
                                    .unwrap_or(json!(null)),
                                None => json!(null),
                            }
                        } else {
                            // Fallback: numeric index or null
                            json!(idx)
                        }
                    }
                };
                (id.clone(), json_val)
            })
            .collect()
    }
}

/// Result that gets serialized to JSON
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
        use serde_json::json;

        let values = state
            .values
            .iter()
            .filter_map(|(id, value)| {
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
                    ElementValue::Choice(Some(idx)) => json!(*idx),
                    _ => return None, // Skip empty text, unselected choice
                };
                Some((id.clone(), json_value))
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
        use serde_json::json;

        // Helper to find element by ID recursively
        fn find_element_by_id<'a>(elements: &'a [Element], id: &str) -> Option<&'a Element> {
            for element in elements {
                match element {
                    e @ Element::Slider { id: eid, .. } if eid == id => return Some(e),
                    e @ Element::Check { id: eid, .. } if eid == id => return Some(e),
                    e @ Element::Input { id: eid, .. } if eid == id => return Some(e),
                    e @ Element::Multi { id: eid, .. } if eid == id => return Some(e),
                    e @ Element::Select { id: eid, .. } if eid == id => return Some(e),
                    Element::Group {
                        elements: children, ..
                    } => {
                        if let Some(e) = find_element_by_id(children, id) {
                            return Some(e);
                        }
                    }
                    // Search in reveals for Checkbox
                    Element::Check { reveals, .. } => {
                        if !reveals.is_empty() {
                            if let Some(e) = find_element_by_id(reveals, id) {
                                return Some(e);
                            }
                        }
                    }
                    Element::Slider { .. } | Element::Input { .. } => {
                        // Slider and Input have no reveals or option_children
                    }
                    // Search in both reveals and option_children for Multiselect and Choice
                    Element::Multi {
                        reveals,
                        option_children,
                        ..
                    }
                    | Element::Select {
                        reveals,
                        option_children,
                        ..
                    } => {
                        // Search reveals first
                        if !reveals.is_empty() {
                            if let Some(e) = find_element_by_id(reveals, id) {
                                return Some(e);
                            }
                        }
                        // Then search option_children
                        if !option_children.is_empty() {
                            for child_elements in option_children.values() {
                                if let Some(e) = find_element_by_id(child_elements, id) {
                                    return Some(e);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            None
        }

        let mut values = HashMap::new();

        for (id, value) in &state.values {
            let element = find_element_by_id(&definition.elements, id);

            let json_value = match (value, element) {
                (ElementValue::Number(n), Some(Element::Slider { .. })) => {
                    json!(*n as i32)
                }
                (ElementValue::Boolean(b), _) => json!(*b),
                (ElementValue::Text(s), _) if !s.is_empty() => json!(s),
                (ElementValue::MultiChoice(selections), Some(Element::Multi { options, .. })) => {
                    let selected: Vec<&str> = selections
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &sel)| sel.then_some(options.get(i).map(|o| o.value())))
                        .flatten()
                        .collect();
                    json!(selected)
                }
                (ElementValue::Choice(Some(idx)), Some(Element::Select { options, .. })) => options
                    .get(*idx)
                    .map(|opt| json!(opt.value()))
                    .unwrap_or(json!(null)),
                (ElementValue::Choice(None), _) => continue,
                (ElementValue::Number(n), _) => json!(*n as i32),
                (ElementValue::MultiChoice(selections), _) => {
                    let indices: Vec<usize> = selections
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &selected)| selected.then_some(i))
                        .collect();
                    json!(indices)
                }
                _ => continue,
            };

            values.insert(id.clone(), json_value);
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
        active_ids: &[String],
    ) -> Self {
        use serde_json::json;

        // Helper to find element by ID recursively (reuse from from_state_with_context)
        fn find_element_by_id<'a>(elements: &'a [Element], id: &str) -> Option<&'a Element> {
            for element in elements {
                match element {
                    e @ Element::Slider { id: eid, .. } if eid == id => return Some(e),
                    e @ Element::Check { id: eid, .. } if eid == id => return Some(e),
                    e @ Element::Input { id: eid, .. } if eid == id => return Some(e),
                    e @ Element::Multi { id: eid, .. } if eid == id => return Some(e),
                    e @ Element::Select { id: eid, .. } if eid == id => return Some(e),
                    Element::Group {
                        elements: children, ..
                    } => {
                        if let Some(e) = find_element_by_id(children, id) {
                            return Some(e);
                        }
                    }
                    // Search in reveals for Checkbox
                    Element::Check { reveals, .. } => {
                        if !reveals.is_empty() {
                            if let Some(e) = find_element_by_id(reveals, id) {
                                return Some(e);
                            }
                        }
                    }
                    Element::Slider { .. } | Element::Input { .. } => {
                        // Slider and Input have no reveals or option_children
                    }
                    // Search in both reveals and option_children for Multiselect and Choice
                    Element::Multi {
                        reveals,
                        option_children,
                        ..
                    }
                    | Element::Select {
                        reveals,
                        option_children,
                        ..
                    } => {
                        // Search reveals first
                        if !reveals.is_empty() {
                            if let Some(e) = find_element_by_id(reveals, id) {
                                return Some(e);
                            }
                        }
                        // Then search option_children
                        if !option_children.is_empty() {
                            for child_elements in option_children.values() {
                                if let Some(e) = find_element_by_id(child_elements, id) {
                                    return Some(e);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            None
        }

        let mut values = HashMap::new();

        for (id, value) in &state.values {
            // Skip this value if it's not in the active elements
            if !active_ids.contains(id) {
                continue;
            }

            let element = find_element_by_id(&definition.elements, id);

            let json_value = match (value, element) {
                (ElementValue::Number(n), Some(Element::Slider { .. })) => {
                    json!(*n as i32)
                }
                (ElementValue::Boolean(b), _) => json!(*b),
                (ElementValue::Text(s), _) if !s.is_empty() => json!(s),
                (ElementValue::MultiChoice(selections), Some(Element::Multi { options, .. })) => {
                    let selected: Vec<&str> = selections
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &sel)| sel.then_some(options.get(i).map(|o| o.value())))
                        .flatten()
                        .collect();
                    json!(selected)
                }
                (ElementValue::Choice(Some(idx)), Some(Element::Select { options, .. })) => options
                    .get(*idx)
                    .map(|opt| json!(opt.value()))
                    .unwrap_or(json!(null)),
                (ElementValue::Choice(None), _) => continue,
                (ElementValue::Number(n), _) => json!(*n as i32),
                (ElementValue::MultiChoice(selections), _) => {
                    let indices: Vec<usize> = selections
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &selected)| selected.then_some(i))
                        .collect();
                    json!(indices)
                }
                _ => continue,
            };

            values.insert(id.clone(), json_value);
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
