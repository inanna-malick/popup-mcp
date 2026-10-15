//! Visibility computation: which elements are currently shown, and which
//! values when-clauses may see.
//!
//! When-clauses are **visibility-scoped**: a condition only sees the values
//! of elements that are themselves visible. An element hidden inside an
//! untaken branch reads as unset (falsy, unselected, count 0), even if it
//! has a default. This matches the result payload's "absent = branch not
//! taken" semantics and means conditions never need to repeat their
//! ancestors' conditions.
//!
//! Visibility and the value map are mutually dependent (hiding an element
//! removes its value, which can change other conditions), so we iterate to
//! a fixed point with a small cap. Pathological mutually-negating
//! conditions stop at the cap rather than looping.

use crate::{evaluate_condition, parse_condition, Element, PopupState};
use serde_json::Value;
use std::collections::HashMap;

const MAX_ITERATIONS: usize = 8;

/// Compute the active (visible) element ids and the value map that
/// when-clauses see, at fixed point.
pub fn compute_visibility(
    elements: &[Element],
    state: &PopupState,
) -> (Vec<String>, HashMap<String, Value>) {
    let full = state.to_value_map(elements);
    let mut active = collect_active(elements, state, &full);

    for _ in 0..MAX_ITERATIONS {
        let filtered: HashMap<String, Value> = full
            .iter()
            .filter(|(id, _)| active.contains(*id))
            .map(|(id, v)| (id.clone(), v.clone()))
            .collect();
        let next = collect_active(elements, state, &filtered);
        if next == active {
            return (active, filtered);
        }
        active = next;
    }

    let filtered = full
        .into_iter()
        .filter(|(id, _)| active.contains(id))
        .collect();
    (active, filtered)
}

/// The active element ids at fixed point (for result construction)
pub fn collect_active_ids(elements: &[Element], state: &PopupState) -> Vec<String> {
    compute_visibility(elements, state).0
}

/// The value map that when-clauses see at fixed point (for rendering)
pub fn visible_value_map(elements: &[Element], state: &PopupState) -> HashMap<String, Value> {
    compute_visibility(elements, state).1
}

/// Evaluate a when-clause against a value map (fail-open on parse errors;
/// definitions are validated before rendering)
pub fn when_is_visible(when: Option<&str>, values: &HashMap<String, Value>) -> bool {
    match when {
        None => true,
        Some(when_expr) => match parse_condition(when_expr) {
            Ok(ast) => evaluate_condition(&ast, values),
            Err(_) => true,
        },
    }
}

/// One structural pass: walk the tree, collecting ids of elements that are
/// visible given branch/reveal state and when-clauses evaluated against
/// `values`
fn collect_active(
    elements: &[Element],
    state: &PopupState,
    values: &HashMap<String, Value>,
) -> Vec<String> {
    let mut active_ids = Vec::new();
    collect_into(elements, state, values, &mut active_ids);
    active_ids
}

fn collect_into(
    elements: &[Element],
    state: &PopupState,
    values: &HashMap<String, Value>,
    active_ids: &mut Vec<String>,
) {
    for element in elements {
        if !when_is_visible(element.when(), values) {
            continue;
        }
        match element {
            Element::Slider { id, .. } | Element::Input { id, .. } | Element::Rank { id, .. } => {
                active_ids.push(id.clone());
            }
            Element::Check { id, reveals, .. } => {
                active_ids.push(id.clone());
                if state.get_boolean(id) && !reveals.is_empty() {
                    collect_into(reveals, state, values, active_ids);
                }
            }
            Element::Multi {
                id,
                options,
                option_children,
                reveals,
                ..
            } => {
                active_ids.push(id.clone());
                if let Some(selections) = state.get_multichoice(id) {
                    for (i, option) in options.iter().enumerate() {
                        if i < selections.len() && selections[i] {
                            if let Some(children) = option_children.get(option.value()) {
                                collect_into(children, state, values, active_ids);
                            }
                        }
                    }
                    if selections.iter().any(|&s| s) && !reveals.is_empty() {
                        collect_into(reveals, state, values, active_ids);
                    }
                }
            }
            Element::Select {
                id,
                options,
                option_children,
                reveals,
                ..
            } => {
                active_ids.push(id.clone());
                let selected_idx = state.get_choice(id).flatten();
                if let Some(idx) = selected_idx {
                    if let Some(option) = options.get(idx) {
                        if let Some(children) = option_children.get(option.value()) {
                            collect_into(children, state, values, active_ids);
                        }
                    }
                    if !reveals.is_empty() {
                        collect_into(reveals, state, values, active_ids);
                    }
                }
            }
            Element::Group { elements, .. } | Element::Page { elements, .. } => {
                collect_into(elements, state, values, active_ids);
            }
            Element::Text { id, .. } | Element::Markdown { id, .. } => {
                if let Some(text_id) = id {
                    active_ids.push(text_id.clone());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PopupDefinition;

    fn parse(json: &str) -> PopupDefinition {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_hidden_default_does_not_leak_into_conditions() {
        // "mode" defaults to Pro but lives behind an unchecked branch; the
        // dependent text must NOT fire while mode is hidden
        let def = parse(
            r#"{
            "title": "T",
            "elements": [
                {"check": "Show more", "id": "show_more"},
                {"select": "Mode", "id": "mode", "options": ["Basic", "Pro"],
                 "default": "Pro", "when": "show_more"},
                {"text": "pro!", "id": "pro_text", "when": "selected(mode, Pro)"}
            ]
        }"#,
        );
        let mut state = PopupState::new(&def);

        let (active, values) = compute_visibility(&def.elements, &state);
        assert!(active.contains(&"show_more".to_string()));
        assert!(!active.contains(&"mode".to_string()));
        // The hidden select's value is excluded from the condition map...
        assert!(!values.contains_key("mode"));
        // ...so the dependent text is hidden even though mode's default is Pro
        assert!(!active.contains(&"pro_text".to_string()));

        // Reveal the branch: mode (defaulted to Pro) now drives the text
        *state.get_boolean_mut("show_more").unwrap() = true;
        let (active, _) = compute_visibility(&def.elements, &state);
        assert!(active.contains(&"mode".to_string()));
        assert!(active.contains(&"pro_text".to_string()));
    }

    #[test]
    fn test_no_ancestor_condition_needed_for_branch_ids() {
        // Referencing an id inside an untaken option-as-key branch is safe
        // without repeating the ancestor's condition
        let def = parse(
            r#"{
            "title": "T",
            "elements": [
                {"select": "Strategy", "id": "strategy", "options": ["A", "B"],
                 "B": [{"select": "Creds", "id": "creds", "options": ["X", "Y"], "default": "Y"}]},
                {"text": "warn", "id": "warn", "when": "selected(creds, Y)"}
            ]
        }"#,
        );
        let mut state = PopupState::new(&def);

        // Branch B untaken: creds hidden, warn hidden despite creds default Y
        let (active, _) = compute_visibility(&def.elements, &state);
        assert!(!active.contains(&"creds".to_string()));
        assert!(!active.contains(&"warn".to_string()));

        // Take branch B: creds visible with default Y, warn fires
        *state.get_choice_mut("strategy").unwrap() = Some(1);
        let (active, _) = compute_visibility(&def.elements, &state);
        assert!(active.contains(&"creds".to_string()));
        assert!(active.contains(&"warn".to_string()));
    }

    #[test]
    fn test_negation_sees_hidden_as_falsy() {
        // !hidden_check is true when hidden_check is hidden, even if its
        // (hidden) value is true
        let def = parse(
            r#"{
            "title": "T",
            "elements": [
                {"check": "Gate", "id": "gate"},
                {"check": "Inner", "id": "inner", "default": true, "when": "gate"},
                {"text": "no inner", "id": "no_inner", "when": "!inner"}
            ]
        }"#,
        );
        let state = PopupState::new(&def);

        let (active, _) = compute_visibility(&def.elements, &state);
        // gate unchecked -> inner hidden -> !inner treats it as false -> visible
        assert!(!active.contains(&"inner".to_string()));
        assert!(active.contains(&"no_inner".to_string()));
    }

    #[test]
    fn test_pathological_mutual_negation_terminates() {
        // a.when = !b, b.when = !a with both defaulting true: no stable
        // assignment exists; we just require termination
        let def = parse(
            r#"{
            "title": "T",
            "elements": [
                {"check": "A", "id": "a", "default": true, "when": "!b"},
                {"check": "B", "id": "b", "default": true, "when": "!a"}
            ]
        }"#,
        );
        let state = PopupState::new(&def);
        let (_active, _values) = compute_visibility(&def.elements, &state);
        // Reaching here without hanging is the assertion
    }
}
