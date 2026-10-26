pub mod condition;
pub mod convo;
pub mod convo_route;
pub mod convo_validate;
pub mod convo_walk;
mod element_deser;
pub mod route;
pub mod validate;
pub mod visibility;

#[cfg(test)]
mod tests;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub use condition::{collect_refs, evaluate_condition, parse_condition, ConditionExpr};
pub use element_deser::label_to_snake_case;
pub use route::{route_packet, RouteAnswer, RouteDecision, RoutePacket};
pub use validate::{validate_definition, ValidationIssue};
pub use visibility::{collect_active_ids, compute_visibility, visible_value_map, when_is_visible};

/// IDs that would collide with keys in the result payload
pub const RESERVED_IDS: &[&str] = &["status", "button", "error", "_notes", "_labels", "_route"];

/// Edge target meaning "finish the popup" instead of another page
pub const SUBMIT_TARGET: &str = "submit";

/// A way out of a page in a decision tree. Either a mechanical `when`
/// clause (evaluated in code) or a semantic `if` description (judged by
/// Jev over the answers so far), never both.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Edge {
    /// Target page id, or "submit" to finish
    pub to: String,
    /// Condition described in words, in terms of the page's answers
    #[serde(default, rename = "if", skip_serializing_if = "Option::is_none")]
    pub r#if: Option<String>,
    /// Mechanical condition (same syntax as element `when`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,
}

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

/// Rendering style for single-select elements
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SelectStyle {
    /// All options visible as radio buttons (best for conversation trees)
    Radio,
    /// Compact dropdown (best for long option lists)
    Dropdown,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PopupDefinition {
    pub title: String,
    pub elements: Vec<Element>,
    /// Custom label for the submit button (defaults to "Submit")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub submit: Option<String>,
    /// Show a free-form "Add note" field so users can say something the
    /// popup didn't anticipate (returned as "_notes"). Defaults to true.
    #[serde(default = "default_true")]
    pub notes: bool,
}

impl PopupDefinition {
    pub fn new(title: impl Into<String>, elements: Vec<Element>) -> Self {
        PopupDefinition {
            title: title.into(),
            elements,
            submit: None,
            notes: true,
        }
    }

    /// Get the effective title
    pub fn effective_title(&self) -> &str {
        &self.title
    }

    /// Get the effective submit button label
    pub fn submit_label(&self) -> &str {
        self.submit.as_deref().unwrap_or("Submit")
    }
}

/// Element types using element-as-key pattern
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
        /// Step increment (defaults to 1). Results are integers when the
        /// step and value are whole numbers.
        step: Option<f32>,
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
        /// Prefilled text - use for accept-or-correct interactions
        default: Option<String>,
        rows: Option<u32>,
        when: Option<String>,
    },

    /// Multiple selection from options (with option-as-key nesting)
    Multi {
        multi: String, // Label text becomes the discriminator key
        id: String,
        options: Vec<OptionValue>,
        /// Option texts pre-checked on open (empty = nothing checked)
        default: Vec<String>,
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
        /// Rendering style override; auto-selected when omitted
        style: Option<SelectStyle>,
        // Option-as-key nesting: HashMap<option_value, Vec<Element>>
        // Custom serialize/deserialize handles option children as direct JSON keys
        option_children: HashMap<String, Vec<Element>>,
        reveals: Vec<Element>,
        when: Option<String>,
    },

    /// Rank options by clicking them in priority order (partial ranking
    /// allowed - unranked options are omitted from the result)
    Rank {
        rank: String, // Label text becomes the discriminator key
        id: String,
        options: Vec<OptionValue>,
        when: Option<String>,
    },

    /// Labeled container for grouping elements
    Group {
        group: String, // Label text becomes the discriminator key
        id: Option<String>,
        elements: Vec<Element>,
        when: Option<String>,
    },

    /// Wizard step (top-level only). When any page is present, every
    /// top-level element must be a page; the GUI shows one page at a time
    /// with Back/Next, and Submit on the last page.
    Page {
        page: String, // Step label becomes the discriminator key
        id: Option<String>,
        elements: Vec<Element>,
        /// Pages with a false condition are skipped entirely
        when: Option<String>,
        /// Decision-tree edges out of this page. Any page with edges turns
        /// the popup into a graph: Next resolves an edge instead of moving
        /// to the following page, and a page without edges is terminal.
        next: Vec<Edge>,
    },
}

impl Element {
    /// The element's id, if it has one
    pub fn id(&self) -> Option<&str> {
        match self {
            Element::Text { id, .. }
            | Element::Markdown { id, .. }
            | Element::Group { id, .. }
            | Element::Page { id, .. } => id.as_deref(),
            Element::Slider { id, .. }
            | Element::Check { id, .. }
            | Element::Input { id, .. }
            | Element::Multi { id, .. }
            | Element::Select { id, .. }
            | Element::Rank { id, .. } => Some(id),
        }
    }

    /// The element's `when` visibility clause, if any
    pub fn when(&self) -> Option<&str> {
        match self {
            Element::Text { when, .. }
            | Element::Markdown { when, .. }
            | Element::Slider { when, .. }
            | Element::Check { when, .. }
            | Element::Input { when, .. }
            | Element::Multi { when, .. }
            | Element::Select { when, .. }
            | Element::Rank { when, .. }
            | Element::Group { when, .. }
            | Element::Page { when, .. } => when.as_deref(),
        }
    }

    /// Human-facing label (the discriminator value)
    pub fn label(&self) -> &str {
        match self {
            Element::Text { text, .. } => text,
            Element::Markdown { markdown, .. } => markdown,
            Element::Slider { slider, .. } => slider,
            Element::Check { check, .. } => check,
            Element::Input { input, .. } => input,
            Element::Multi { multi, .. } => multi,
            Element::Select { select, .. } => select,
            Element::Rank { rank, .. } => rank,
            Element::Group { group, .. } => group,
            Element::Page { page, .. } => page,
        }
    }

    /// A page's routing id: explicit id, else derived from its label
    pub fn page_id(&self) -> Option<String> {
        match self {
            Element::Page { page, id, .. } => {
                Some(id.clone().unwrap_or_else(|| label_to_snake_case(page)))
            }
            _ => None,
        }
    }

    /// Element kind name as used in JSON
    pub fn kind(&self) -> &'static str {
        match self {
            Element::Text { .. } => "text",
            Element::Markdown { .. } => "markdown",
            Element::Slider { .. } => "slider",
            Element::Check { .. } => "check",
            Element::Input { .. } => "input",
            Element::Multi { .. } => "multi",
            Element::Select { .. } => "select",
            Element::Rank { .. } => "rank",
            Element::Group { .. } => "group",
            Element::Page { .. } => "page",
        }
    }

    /// All nested element lists (reveals, option children, group members),
    /// regardless of visibility conditions
    pub fn child_lists(&self) -> Vec<&[Element]> {
        match self {
            Element::Text { .. } | Element::Markdown { .. } | Element::Slider { .. } => vec![],
            Element::Input { .. } | Element::Rank { .. } => vec![],
            Element::Check { reveals, .. } => vec![reveals.as_slice()],
            Element::Multi {
                option_children,
                reveals,
                ..
            }
            | Element::Select {
                option_children,
                reveals,
                ..
            } => {
                let mut lists: Vec<&[Element]> = option_children
                    .values()
                    .map(|children| children.as_slice())
                    .collect();
                lists.push(reveals.as_slice());
                lists
            }
            Element::Group { elements, .. } | Element::Page { elements, .. } => {
                vec![elements.as_slice()]
            }
        }
    }
}

/// Find an element by id anywhere in the tree (including nested branches)
pub fn find_element_by_id<'a>(elements: &'a [Element], id: &str) -> Option<&'a Element> {
    for element in elements {
        if element.id() == Some(id) {
            return Some(element);
        }
        for list in element.child_lists() {
            if let Some(found) = find_element_by_id(list, id) {
                return Some(found);
            }
        }
    }
    None
}

/// Visit every element in the tree (including nested branches)
pub fn walk_elements<'a>(elements: &'a [Element], f: &mut impl FnMut(&'a Element)) {
    for element in elements {
        f(element);
        for list in element.child_lists() {
            walk_elements(list, f);
        }
    }
}

/// Unified value type for all widget states
#[derive(Debug, Clone, PartialEq, JsonSchema)]
pub enum ElementValue {
    Number(f32),
    Boolean(bool),
    Text(String),
    MultiChoice(Vec<bool>),
    Choice(Option<usize>),
    /// Option indices in ranked order (may be a subset of the options)
    Order(Vec<usize>),
}

/// Runtime state of the popup
#[derive(Default)]
pub struct PopupState {
    pub values: HashMap<String, ElementValue>, // ID -> value
    pub button_clicked: Option<String>,
    /// Free-form user note (the auto-injected "Add note" field)
    pub notes: String,
}

impl PopupState {
    pub fn new(definition: &PopupDefinition) -> Self {
        let mut state = PopupState::default();
        walk_elements(&definition.elements, &mut |element| match element {
            Element::Slider {
                id,
                min,
                max,
                default,
                ..
            } => {
                let default_value = default.unwrap_or((min + max) / 2.0);
                state
                    .values
                    .insert(id.clone(), ElementValue::Number(default_value));
            }
            Element::Check { id, default, .. } => {
                state
                    .values
                    .insert(id.clone(), ElementValue::Boolean(*default));
            }
            Element::Input { id, default, .. } => {
                state.values.insert(
                    id.clone(),
                    ElementValue::Text(default.clone().unwrap_or_default()),
                );
            }
            Element::Multi {
                id,
                options,
                default,
                ..
            } => {
                state.values.insert(
                    id.clone(),
                    ElementValue::MultiChoice(
                        options
                            .iter()
                            .map(|opt| default.iter().any(|d| d == opt.value()))
                            .collect(),
                    ),
                );
            }
            Element::Select {
                id,
                options,
                default,
                ..
            } => {
                let default_idx = default.as_ref().and_then(|default_val| {
                    options.iter().position(|opt| opt.value() == default_val)
                });
                state
                    .values
                    .insert(id.clone(), ElementValue::Choice(default_idx));
            }
            Element::Rank { id, .. } => {
                state
                    .values
                    .insert(id.clone(), ElementValue::Order(Vec::new()));
            }
            Element::Group { .. }
            | Element::Page { .. }
            | Element::Text { .. }
            | Element::Markdown { .. } => {}
        });
        state
    }

    // Helper methods for GUI access
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

    pub fn get_order_mut(&mut self, id: &str) -> Option<&mut Vec<usize>> {
        match self.values.get_mut(id) {
            Some(ElementValue::Order(ref mut v)) => Some(v),
            _ => None,
        }
    }

    pub fn get_order(&self, id: &str) -> Option<&Vec<usize>> {
        match self.values.get(id) {
            Some(ElementValue::Order(v)) => Some(v),
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
                    ElementValue::Order(order) => {
                        // Array of option texts in ranked order, so
                        // selected() and count() work on rankings
                        if let Some(Element::Rank { options, .. }) =
                            find_element_by_id(elements, id)
                        {
                            let ranked: Vec<&str> = order
                                .iter()
                                .filter_map(|&i| options.get(i).map(|o| o.value()))
                                .collect();
                            json!(ranked)
                        } else {
                            json!(order)
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
}

/// Render a slider value: integer when whole, float otherwise
fn number_to_json(n: f32) -> Value {
    use serde_json::json;
    if (n - n.round()).abs() < 1e-6 {
        json!(n.round() as i64)
    } else {
        json!(n)
    }
}

impl PopupResult {
    /// Build a completed result from popup state.
    ///
    /// Value semantics (documented in the MCP tool description - keep in sync):
    /// - absent key: the element was never shown (branch not taken)
    /// - "" for inputs: shown but left blank
    /// - null for selects: shown but nothing selected
    /// - selects/multis return option text, not indices
    /// - sliders return integers when the value is whole
    /// - "_notes" carries the free-form user note when non-empty
    /// - "_labels" maps each returned id to its human-facing label, so the
    ///   result is self-documenting without the original definition
    pub fn from_state(
        state: &PopupState,
        definition: &PopupDefinition,
        active_ids: &[String],
    ) -> Self {
        use serde_json::json;

        let mut values = HashMap::new();
        let mut labels = serde_json::Map::new();

        for (id, value) in &state.values {
            if !active_ids.contains(id) {
                continue;
            }

            let element = find_element_by_id(&definition.elements, id);
            if let Some(element) = element {
                labels.insert(id.clone(), json!(element.label()));
            }

            let json_value = match (value, element) {
                (ElementValue::Number(n), _) => number_to_json(*n),
                (ElementValue::Boolean(b), _) => json!(*b),
                (ElementValue::Text(s), _) => json!(s),
                (ElementValue::MultiChoice(selections), Some(Element::Multi { options, .. })) => {
                    let selected: Vec<&str> = selections
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &sel)| {
                            if sel {
                                options.get(i).map(|o| o.value())
                            } else {
                                None
                            }
                        })
                        .collect();
                    json!(selected)
                }
                (ElementValue::MultiChoice(selections), _) => {
                    let indices: Vec<usize> = selections
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &selected)| selected.then_some(i))
                        .collect();
                    json!(indices)
                }
                (ElementValue::Order(order), Some(Element::Rank { options, .. })) => {
                    let ranked: Vec<&str> = order
                        .iter()
                        .filter_map(|&i| options.get(i).map(|o| o.value()))
                        .collect();
                    json!(ranked)
                }
                (ElementValue::Order(order), _) => json!(order),
                (ElementValue::Choice(Some(idx)), Some(Element::Select { options, .. })) => options
                    .get(*idx)
                    .map(|opt| json!(opt.value()))
                    .unwrap_or(json!(null)),
                (ElementValue::Choice(Some(idx)), _) => json!(*idx),
                (ElementValue::Choice(None), _) => json!(null),
            };

            values.insert(id.clone(), json_value);
        }

        if !state.notes.trim().is_empty() {
            values.insert("_notes".to_string(), json!(state.notes.trim()));
        }
        if !labels.is_empty() {
            values.insert("_labels".to_string(), Value::Object(labels));
        }

        PopupResult::Completed {
            values,
            button: state
                .button_clicked
                .clone()
                .unwrap_or_else(|| "submit".to_string()),
        }
    }
}
