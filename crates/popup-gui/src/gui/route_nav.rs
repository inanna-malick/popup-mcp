//! Graph-mode navigation helpers: everything about walking a decision
//! tree of pages that does not need a window, so it can be tested without
//! rendering or network access.

use popup_common::route::PageAnswers;
use popup_common::{
    collect_active_ids, find_element_by_id, walk_elements, Element, PopupDefinition, PopupResult,
    PopupState,
};
use serde_json::{json, Value};
use std::collections::HashMap;

/// A popup is a decision tree when any top-level page declares edges.
/// Pages without edges keep the linear wizard behaviour.
pub fn is_graph(elements: &[Element]) -> bool {
    elements
        .iter()
        .any(|e| matches!(e, Element::Page { next, .. } if !next.is_empty()))
}

/// Page id -> page label, for edge descriptions and panel rows
pub fn page_labels(elements: &[Element]) -> HashMap<String, String> {
    elements
        .iter()
        .filter_map(|e| Some((e.page_id()?, e.label().to_string())))
        .collect()
}

/// Index of the top-level page with this routing id
pub fn page_index(elements: &[Element], to: &str) -> Option<usize> {
    elements
        .iter()
        .position(|e| e.page_id().as_deref() == Some(to))
}

/// Every element id inside one page's subtree
pub fn ids_in_page(page: &Element) -> Vec<String> {
    let mut ids = Vec::new();
    for list in page.child_lists() {
        walk_elements(list, &mut |e| {
            if let Some(id) = e.id() {
                ids.push(id.to_string());
            }
        });
    }
    ids
}

/// Write inferred answers into the destination page's widgets. Returns
/// the ids actually filled, so the GUI can mark them: a prefilled widget
/// opens ticked with a note that it was read from what the person wrote,
/// and one click changes it. Nothing is ever skipped or hidden.
///
/// Only select and multi are prefillable, and only options the widget
/// actually has; anything else is ignored rather than guessed at.
pub fn apply_prefill(
    definition: &PopupDefinition,
    state: &mut PopupState,
    page: &Element,
    prefill: &HashMap<String, (String, f64)>,
) -> HashMap<String, String> {
    let mut applied = HashMap::new();
    for id in ids_in_page(page) {
        let Some((option, _)) = prefill.get(&id) else {
            continue;
        };
        let Some(element) = find_element_by_id(&definition.elements, &id) else {
            continue;
        };
        let options = match element {
            Element::Select { options, .. } | Element::Multi { options, .. } => options,
            _ => continue,
        };
        let Some(idx) = options.iter().position(|o| o.value() == option) else {
            continue;
        };
        match element {
            Element::Select { .. } => {
                if let Some(choice) = state.get_choice_mut(&id) {
                    *choice = Some(idx);
                    applied.insert(id.clone(), option.clone());
                }
            }
            Element::Multi { .. } => {
                if let Some(flags) = state.get_multichoice_mut(&id) {
                    if let Some(flag) = flags.get_mut(idx) {
                        *flag = true;
                        applied.insert(id.clone(), option.clone());
                    }
                }
            }
            _ => {}
        }
    }
    applied
}

/// Active ids restricted to the pages actually visited. In graph mode an
/// unvisited page's elements were never shown, so their defaults must not
/// reach the result (same rule as "absent = branch not taken").
pub fn visited_active_ids(
    definition: &PopupDefinition,
    state: &PopupState,
    visited: &[usize],
) -> Vec<String> {
    let active = collect_active_ids(&definition.elements, state);
    let in_visited: Vec<String> = visited
        .iter()
        .filter_map(|&i| definition.elements.get(i))
        .flat_map(ids_in_page)
        .collect();
    active
        .into_iter()
        .filter(|id| in_visited.contains(id))
        .collect()
}

/// The path so far as Jev sees it: one entry per visited page, answers
/// keyed by human-facing label (the edge conditions are written in words,
/// so the state has to speak the same language), visible values only.
pub fn build_path(
    definition: &PopupDefinition,
    state: &PopupState,
    visited: &[usize],
    page_notes: &HashMap<usize, String>,
) -> Vec<PageAnswers> {
    let active = collect_active_ids(&definition.elements, state);
    let values = state.to_value_map(&definition.elements);

    visited
        .iter()
        .filter_map(|&idx| {
            let page = definition.elements.get(idx)?;
            let answers: HashMap<String, Value> = ids_in_page(page)
                .into_iter()
                .filter(|id| active.contains(id))
                .filter_map(|id| {
                    let element = find_element_by_id(&definition.elements, &id)?;
                    Some((
                        element.label().to_string(),
                        with_descriptions(element, values.get(&id)?),
                    ))
                })
                .collect();
            let note = page_notes
                .get(&idx)
                .map(|n| n.trim())
                .filter(|n| !n.is_empty())
                .map(str::to_string);
            Some(PageAnswers {
                page: page.label().to_string(),
                answers,
                note,
            })
        })
        .collect()
}

/// An option's text is only half of what the user read: the description
/// under it is the part that says what the option means. The conditions
/// are judged against this state, so an answer travels with the words
/// that were on screen beside it, not just the label that was clicked.
pub fn with_descriptions(element: &Element, value: &Value) -> Value {
    let options = match element {
        Element::Select { options, .. } | Element::Multi { options, .. } => options,
        _ => return value.clone(),
    };
    let describe = |chosen: &str| -> Value {
        match options
            .iter()
            .find(|o| o.value() == chosen)
            .and_then(|o| o.description())
        {
            Some(desc) => json!(format!("{} ({})", chosen, desc)),
            None => json!(chosen),
        }
    };
    match value {
        Value::String(s) => describe(s),
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|i| i.as_str().map(describe).unwrap_or_else(|| i.clone()))
                .collect(),
        ),
        other => other.clone(),
    }
}

/// Attach the routing log to a completed result under `_route`
pub fn with_route_log(result: PopupResult, route_log: &[Value]) -> PopupResult {
    match result {
        PopupResult::Completed { mut values, button } => {
            if !route_log.is_empty() {
                values.insert("_route".to_string(), Value::Array(route_log.to_vec()));
            }
            PopupResult::Completed { values, button }
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use popup_common::ElementValue;
    use serde_json::json;

    fn tree() -> PopupDefinition {
        let json = r#"{
          "title": "Incident triage",
          "elements": [
            {"page": "Symptom", "elements": [
              {"select": "What broke", "id": "symptom", "options": ["Checkout fails", "Dashboard slow"]}
            ], "next": [
              {"to": "outage", "if": "the user reports something users cannot complete at all"},
              {"to": "perf", "if": "the user reports something working but degraded"}
            ]},
            {"page": "Outage", "id": "outage", "elements": [
              {"check": "Roll back the last deploy", "id": "rollback", "default": true}
            ], "next": [{"to": "submit", "if": "the user decided what to do"}]},
            {"page": "Performance", "id": "perf", "elements": [
              {"slider": "Latency (s)", "id": "latency", "min": 1, "max": 30, "default": 5}
            ]}
          ]
        }"#;
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn graph_mode_only_when_edges_exist() {
        let def = tree();
        assert!(is_graph(&def.elements));
        let wizard: PopupDefinition = serde_json::from_str(
            r#"{"title":"W","elements":[{"page":"A","elements":[]},{"page":"B","elements":[]}]}"#,
        )
        .unwrap();
        assert!(!is_graph(&wizard.elements));
    }

    #[test]
    fn page_ids_are_explicit_or_derived() {
        let def = tree();
        let labels = page_labels(&def.elements);
        assert_eq!(labels.get("symptom_"), None);
        assert_eq!(labels.get("outage").map(String::as_str), Some("Outage"));
        // derived from the label when no id is given
        assert_eq!(labels.get("symptom").map(String::as_str), Some("Symptom"));
        assert_eq!(page_index(&def.elements, "perf"), Some(2));
        assert_eq!(page_index(&def.elements, "nope"), None);
    }

    #[test]
    fn path_uses_labels_and_visited_pages_only() {
        let def = tree();
        let mut state = PopupState::new(&def);
        state
            .values
            .insert("symptom".into(), ElementValue::Choice(Some(0)));
        let notes = HashMap::from([(0usize, "  card form spins forever  ".to_string())]);

        let path = build_path(&def, &state, &[0, 1], &notes);
        assert_eq!(path.len(), 2);
        assert_eq!(path[0].page, "Symptom");
        assert_eq!(path[0].answers["What broke"], json!("Checkout fails"));
        assert_eq!(path[0].note.as_deref(), Some("card form spins forever"));
        assert_eq!(path[1].answers["Roll back the last deploy"], json!(true));
        // the unvisited performance page contributes nothing
        assert!(path.iter().all(|p| !p.answers.contains_key("Latency (s)")));
    }

    #[test]
    fn unvisited_pages_do_not_reach_the_result() {
        let def = tree();
        let state = PopupState::new(&def);
        let ids = visited_active_ids(&def, &state, &[0, 1]);
        assert!(ids.contains(&"symptom".to_string()));
        assert!(ids.contains(&"rollback".to_string()));
        assert!(
            !ids.contains(&"latency".to_string()),
            "unvisited page's slider default leaked: {:?}",
            ids
        );
    }

    #[test]
    fn route_log_rides_in_the_result() {
        let def = tree();
        let state = PopupState::new(&def);
        let ids = visited_active_ids(&def, &state, &[0]);
        let base = PopupResult::from_state(&state, &def, &ids);
        let log = vec![json!({"from": "symptom", "to": "outage", "via": "jev", "confidence": 0.9})];
        let PopupResult::Completed { values, .. } = with_route_log(base, &log) else {
            panic!("expected completed");
        };
        assert_eq!(values["_route"][0]["to"], json!("outage"));
    }

    #[test]
    fn cancelled_results_keep_no_route_log() {
        let log = vec![json!({"from": "a", "to": "b"})];
        assert!(matches!(
            with_route_log(PopupResult::Cancelled, &log),
            PopupResult::Cancelled
        ));
    }
}
