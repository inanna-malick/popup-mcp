//! Pre-flight validation for popup definitions.
//!
//! Runs before any window is shown so structural mistakes come back to the
//! caller as actionable errors instead of silently mis-rendering.

use crate::{collect_refs, parse_condition, walk_elements, Element, PopupDefinition, RESERVED_IDS};
use std::collections::HashMap;

/// A single validation problem, with enough context to fix it
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ValidationIssue {
    /// Which element the problem is on (kind + label)
    pub element: String,
    pub message: String,
}

impl std::fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.element, self.message)
    }
}

/// Levenshtein distance, used for "did you mean" suggestions
pub fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

/// Find the closest candidate to `input`, if it's close enough to be a
/// plausible typo
pub fn suggest<'a>(input: &str, candidates: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    candidates
        .map(|c| (edit_distance(&input.to_lowercase(), &c.to_lowercase()), c))
        .filter(|(d, c)| *d <= (c.len() / 3).max(2))
        .min_by_key(|(d, _)| *d)
        .map(|(_, c)| c)
}

const WHEN_SYNTAX_HINT: &str = "when-clause syntax: bare ids are truthy checks (\"advanced\"), \
selected(id, value) tests a specific option (quote the value if it contains a closing paren: \
selected(mode, \"Fast (beta)\")), count(id) > N counts selections, \
combine with && || ! and parentheses";

/// Field names reserved on select elements (an option value equal to one of
/// these could not be used as a branch key)
pub(crate) const SELECT_FIELDS: &[&str] = &[
    "select", "id", "options", "default", "style", "reveals", "when",
];

/// Field names reserved on multi elements
pub(crate) const MULTI_FIELDS: &[&str] = &["multi", "id", "options", "default", "reveals", "when"];

/// The auto-injected catch-all option label (see popup-gui transform)
pub const OTHER_LABEL: &str = "Other (please specify)";

/// Collect (field_id, literal_value) pairs from every selected() call
fn collect_selected(expr: &crate::ConditionExpr, out: &mut Vec<(String, String)>) {
    use crate::ConditionExpr as E;
    match expr {
        E::Or(exprs) | E::And(exprs) | E::Any(exprs) | E::All(exprs) => {
            exprs.iter().for_each(|e| collect_selected(e, out))
        }
        E::Not(inner) | E::Count(inner) => collect_selected(inner, out),
        E::Compare { left, right, .. } => {
            collect_selected(left, out);
            collect_selected(right, out);
        }
        E::Selected(field, value) => {
            if let (E::Ref(id), E::String(v)) = (&**field, &**value) {
                out.push((id.clone(), v.clone()));
            }
        }
        E::Ref(_) | E::Number(_) | E::String(_) | E::Boolean(_) => {}
    }
}

/// Validate a popup definition. Returns an empty vec when the definition is
/// sound; otherwise one issue per problem, each phrased to be fixable.
pub fn validate_definition(def: &PopupDefinition) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // Collect every element and its id
    let mut all: Vec<&Element> = Vec::new();
    walk_elements(&def.elements, &mut |e| all.push(e));

    let mut id_counts: HashMap<&str, usize> = HashMap::new();
    let mut elements_by_id: HashMap<&str, &Element> = HashMap::new();
    for element in &all {
        if let Some(id) = element.id() {
            *id_counts.entry(id).or_insert(0) += 1;
            elements_by_id.entry(id).or_insert(element);
        }
    }

    let issue = |element: &Element, message: String| ValidationIssue {
        element: format!("{} \"{}\"", element.kind(), element.label()),
        message,
    };

    // Page placement: pages are wizard steps, so they only make sense as the
    // complete top level - mixing loose elements with pages is ambiguous
    let page_count = def
        .elements
        .iter()
        .filter(|e| matches!(e, Element::Page { .. }))
        .count();
    if page_count > 0 && page_count < def.elements.len() {
        for element in &def.elements {
            if !matches!(element, Element::Page { .. }) {
                issues.push(issue(
                    element,
                    "popups with pages must have ONLY pages at the top level - move this element \
                     inside a page"
                        .to_string(),
                ));
            }
        }
    }
    for top in &def.elements {
        for list in top.child_lists() {
            let mut nested_pages: Vec<&Element> = Vec::new();
            walk_elements(list, &mut |e| {
                if matches!(e, Element::Page { .. }) {
                    nested_pages.push(e);
                }
            });
            for page in nested_pages {
                issues.push(issue(
                    page,
                    "pages can only appear at the top level of the popup - use a group for \
                     nested sections"
                        .to_string(),
                ));
            }
        }
    }

    // Decision-tree edges: every target must be a page id or "submit"
    let page_ids: Vec<String> = def.elements.iter().filter_map(|e| e.page_id()).collect();
    for element in &def.elements {
        let Element::Page { next, .. } = element else {
            continue;
        };
        for edge in next {
            if edge.to == crate::SUBMIT_TARGET || page_ids.iter().any(|p| p == &edge.to) {
                continue;
            }
            let hint = suggest(&edge.to, page_ids.iter().map(String::as_str))
                .map(|s| format!(" - did you mean \"{}\"?", s))
                .unwrap_or_default();
            issues.push(issue(
                element,
                format!(
                    "edge target \"{}\" is not a page id{} (page ids: {}; or \"submit\" to finish)",
                    edge.to,
                    hint,
                    page_ids.join(", ")
                ),
            ));
        }
    }

    for element in &all {
        // Duplicate and reserved ids
        if let Some(id) = element.id() {
            if id_counts[id] > 1 {
                // Reported on each occurrence so the caller can find both
                let extra = if id.ends_with("_other_text") {
                    " (ids ending in \"_other_text\" are auto-generated for the injected \
                     'Other' option's text input - pick a different id)"
                } else {
                    " (note: omitted ids are auto-derived from labels, so identical labels collide)"
                };
                issues.push(issue(
                    element,
                    format!(
                        "duplicate id \"{}\" ({} elements share it) - ids must be unique across \
                         the whole popup, even in mutually exclusive branches{}",
                        id, id_counts[id], extra
                    ),
                ));
            }
            if RESERVED_IDS.contains(&id) {
                issues.push(issue(
                    element,
                    format!(
                        "id \"{}\" is reserved (result payload uses it) - pick another id",
                        id
                    ),
                ));
            }
        }

        // Per-kind sanity checks
        match element {
            Element::Slider {
                min,
                max,
                default,
                step,
                ..
            } => {
                if min >= max {
                    issues.push(issue(
                        element,
                        format!("min ({}) must be less than max ({})", min, max),
                    ));
                }
                if let Some(d) = default {
                    if d < min || d > max {
                        issues.push(issue(
                            element,
                            format!("default ({}) is outside [{}, {}]", d, min, max),
                        ));
                    }
                }
                if let Some(s) = step {
                    if *s <= 0.0 {
                        issues.push(issue(element, format!("step ({}) must be positive", s)));
                    }
                }
            }
            Element::Select {
                options, default, ..
            } => {
                if let Some(d) = default {
                    if !options.iter().any(|o| o.value() == d) {
                        let hint = suggest(d, options.iter().map(|o| o.value()))
                            .map(|s| format!(" - did you mean \"{}\"?", s))
                            .unwrap_or_default();
                        issues.push(issue(
                            element,
                            format!("default \"{}\" is not one of the options{}", d, hint),
                        ));
                    }
                }
                if options.is_empty() {
                    issues.push(issue(element, "options must not be empty".to_string()));
                }
                check_option_field_collisions(element, options, SELECT_FIELDS, &mut issues);
            }
            Element::Rank { options, .. } if options.is_empty() => {
                issues.push(issue(element, "options must not be empty".to_string()));
            }
            Element::Multi {
                options, default, ..
            } => {
                if options.is_empty() {
                    issues.push(issue(element, "options must not be empty".to_string()));
                }
                for d in default {
                    if !options.iter().any(|o| o.value() == d) {
                        let hint = suggest(d, options.iter().map(|o| o.value()))
                            .map(|s| format!(" - did you mean \"{}\"?", s))
                            .unwrap_or_default();
                        issues.push(issue(
                            element,
                            format!("default \"{}\" is not one of the options{}", d, hint),
                        ));
                    }
                }
                check_option_field_collisions(element, options, MULTI_FIELDS, &mut issues);
            }
            _ => {}
        }

        // When-clause checks: must parse, and every referenced id must exist
        if let Some(when_expr) = element.when() {
            match parse_condition(when_expr) {
                Err(e) => {
                    issues.push(issue(
                        element,
                        format!(
                            "when clause \"{}\" failed to parse: {}. {}",
                            when_expr, e, WHEN_SYNTAX_HINT
                        ),
                    ));
                }
                Ok(ast) => {
                    let mut refs = Vec::new();
                    collect_refs(&ast, &mut refs);
                    for r in refs {
                        if !id_counts.contains_key(r.as_str()) {
                            let hint = suggest(&r, id_counts.keys().copied())
                                .map(|s| format!(" - did you mean \"{}\"?", s))
                                .unwrap_or_default();
                            issues.push(issue(
                                element,
                                format!(
                                    "when clause \"{}\" references unknown id \"{}\"{}",
                                    when_expr, r, hint
                                ),
                            ));
                        }
                    }

                    // selected() must target a select/multi, and the value
                    // must be one of its options - a typo here would make
                    // the condition silently always-false
                    let mut selected_pairs = Vec::new();
                    collect_selected(&ast, &mut selected_pairs);
                    for (field, value) in selected_pairs {
                        match elements_by_id.get(field.as_str()) {
                            None => {} // unknown id already reported above
                            Some(Element::Select { options, .. })
                            | Some(Element::Multi { options, .. })
                            | Some(Element::Rank { options, .. }) => {
                                let is_other = value.eq_ignore_ascii_case("other")
                                    || value.eq_ignore_ascii_case(OTHER_LABEL);
                                if !is_other && !options.iter().any(|o| o.value() == value) {
                                    let hint = suggest(&value, options.iter().map(|o| o.value()))
                                        .map(|s| format!(" - did you mean \"{}\"?", s))
                                        .unwrap_or_default();
                                    issues.push(issue(
                                        element,
                                        format!(
                                            "when clause \"{}\": \"{}\" is not an option of \"{}\" \
                                             (options: {}){}",
                                            when_expr,
                                            value,
                                            field,
                                            options
                                                .iter()
                                                .map(|o| o.value())
                                                .collect::<Vec<_>>()
                                                .join(", "),
                                            hint
                                        ),
                                    ));
                                }
                            }
                            Some(target) => {
                                let fix = match target {
                                    Element::Check { .. } => format!(
                                        " - for checkboxes use the bare id (\"{}\" or \"!{}\")",
                                        field, field
                                    ),
                                    _ => String::new(),
                                };
                                issues.push(issue(
                                    element,
                                    format!(
                                        "when clause \"{}\": selected() only works on select/multi/rank \
                                         elements, but \"{}\" is a {}{}",
                                        when_expr,
                                        field,
                                        target.kind(),
                                        fix
                                    ),
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    issues
}

/// An option value equal to a field name could never be used as a branch key
/// (the deserializer reads it as the field), so reject it up front
fn check_option_field_collisions(
    element: &Element,
    options: &[crate::OptionValue],
    fields: &[&str],
    issues: &mut Vec<ValidationIssue>,
) {
    for option in options {
        if fields.contains(&option.value()) {
            issues.push(ValidationIssue {
                element: format!("{} \"{}\"", element.kind(), element.label()),
                message: format!(
                    "option \"{}\" collides with the \"{}\" field name, so it could never be \
                     used as a branch key - rename the option (e.g. capitalize it)",
                    option.value(),
                    option.value()
                ),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Element, OptionValue};
    use std::collections::HashMap;

    fn slider(id: &str, min: f32, max: f32, default: Option<f32>) -> Element {
        Element::Slider {
            slider: id.to_string(),
            id: id.to_string(),
            min,
            max,
            default,
            step: None,
            when: None,
        }
    }

    #[test]
    fn test_valid_definition_passes() {
        let def = PopupDefinition::new(
            "Test",
            vec![
                Element::Check {
                    check: "Advanced".to_string(),
                    id: "advanced".to_string(),
                    default: false,
                    reveals: vec![],
                    when: None,
                },
                Element::Slider {
                    slider: "Level".to_string(),
                    id: "level".to_string(),
                    min: 0.0,
                    max: 10.0,
                    default: Some(5.0),
                    step: None,
                    when: Some("advanced".to_string()),
                },
            ],
        );
        assert_eq!(validate_definition(&def), vec![]);
    }

    #[test]
    fn test_duplicate_ids_flagged() {
        let def = PopupDefinition::new(
            "Test",
            vec![slider("x", 0.0, 10.0, None), slider("x", 0.0, 5.0, None)],
        );
        let issues = validate_definition(&def);
        assert_eq!(issues.len(), 2);
        assert!(issues[0].message.contains("duplicate id"));
    }

    #[test]
    fn test_reserved_id_flagged() {
        let def = PopupDefinition::new("Test", vec![slider("status", 0.0, 10.0, None)]);
        let issues = validate_definition(&def);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("reserved"));
    }

    #[test]
    fn test_slider_range_checks() {
        let def = PopupDefinition::new(
            "Test",
            vec![
                slider("a", 10.0, 0.0, None),
                slider("b", 0.0, 10.0, Some(20.0)),
            ],
        );
        let issues = validate_definition(&def);
        assert_eq!(issues.len(), 2);
        assert!(issues[0].message.contains("less than max"));
        assert!(issues[1].message.contains("outside"));
    }

    #[test]
    fn test_select_default_not_an_option() {
        let def = PopupDefinition::new(
            "Test",
            vec![Element::Select {
                select: "Mode".to_string(),
                id: "mode".to_string(),
                options: vec![
                    OptionValue::Simple("Simple".to_string()),
                    OptionValue::Simple("Advanced".to_string()),
                ],
                default: Some("Advancd".to_string()),
                style: None,
                option_children: HashMap::new(),
                reveals: vec![],
                when: None,
            }],
        );
        let issues = validate_definition(&def);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("did you mean \"Advanced\""));
    }

    #[test]
    fn test_when_unknown_ref_with_suggestion() {
        let def = PopupDefinition::new(
            "Test",
            vec![
                Element::Check {
                    check: "Advanced".to_string(),
                    id: "advanced".to_string(),
                    default: false,
                    reveals: vec![],
                    when: None,
                },
                Element::Text {
                    text: "hi".to_string(),
                    id: None,
                    when: Some("advancedd".to_string()),
                },
            ],
        );
        let issues = validate_definition(&def);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("unknown id \"advancedd\""));
        assert!(issues[0].message.contains("did you mean \"advanced\""));
    }

    #[test]
    fn test_when_parse_error_includes_hint() {
        let def = PopupDefinition::new(
            "Test",
            vec![Element::Text {
                text: "hi".to_string(),
                id: None,
                when: Some("advanced && (".to_string()),
            }],
        );
        let issues = validate_definition(&def);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("failed to parse"));
        assert!(issues[0].message.contains("when-clause syntax"));
    }

    #[test]
    fn test_selected_value_not_treated_as_ref() {
        // selected(mode, Pro): "Pro" is a literal option value, not an id ref
        let def = PopupDefinition::new(
            "Test",
            vec![
                Element::Select {
                    select: "Mode".to_string(),
                    id: "mode".to_string(),
                    options: vec![
                        OptionValue::Simple("Basic".to_string()),
                        OptionValue::Simple("Pro".to_string()),
                    ],
                    default: None,
                    style: None,
                    option_children: HashMap::new(),
                    reveals: vec![],
                    when: None,
                },
                Element::Text {
                    text: "pro!".to_string(),
                    id: None,
                    when: Some("selected(mode, Pro)".to_string()),
                },
            ],
        );
        assert_eq!(validate_definition(&def), vec![]);
    }

    #[test]
    fn test_edit_distance() {
        assert_eq!(edit_distance("abc", "abc"), 0);
        assert_eq!(edit_distance("abc", "abd"), 1);
        assert_eq!(edit_distance("", "abc"), 3);
    }

    fn mode_select() -> Element {
        Element::Select {
            select: "Mode".to_string(),
            id: "mode".to_string(),
            options: vec![
                OptionValue::Simple("Basic".to_string()),
                OptionValue::Simple("Pro".to_string()),
            ],
            default: None,
            style: None,
            option_children: HashMap::new(),
            reveals: vec![],
            when: None,
        }
    }

    #[test]
    fn test_selected_value_must_be_an_option() {
        // A typo'd selected() value would be a silently always-false
        // condition - the same subtree-dropping trap as a typo'd branch key
        let def = PopupDefinition::new(
            "Test",
            vec![
                mode_select(),
                Element::Text {
                    text: "pro!".to_string(),
                    id: None,
                    when: Some("selected(mode, Proo)".to_string()),
                },
            ],
        );
        let issues = validate_definition(&def);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("\"Proo\" is not an option"));
        assert!(issues[0].message.contains("did you mean \"Pro\""));
    }

    #[test]
    fn test_selected_other_is_allowed() {
        // "Other (please specify)" is injected later, so it's always legal
        let def = PopupDefinition::new(
            "Test",
            vec![
                mode_select(),
                Element::Text {
                    text: "custom".to_string(),
                    id: None,
                    when: Some("selected(mode, \"Other (please specify)\")".to_string()),
                },
            ],
        );
        assert_eq!(validate_definition(&def), vec![]);
    }

    #[test]
    fn test_selected_on_checkbox_is_error() {
        let def = PopupDefinition::new(
            "Test",
            vec![
                Element::Check {
                    check: "Debug".to_string(),
                    id: "debug".to_string(),
                    default: false,
                    reveals: vec![],
                    when: None,
                },
                Element::Text {
                    text: "debugging".to_string(),
                    id: None,
                    when: Some("selected(debug, true)".to_string()),
                },
            ],
        );
        let issues = validate_definition(&def);
        assert_eq!(issues.len(), 1);
        assert!(issues[0]
            .message
            .contains("selected() only works on select/multi"));
        assert!(issues[0].message.contains("use the bare id"));
    }

    #[test]
    fn test_option_value_colliding_with_field_name() {
        // {"select": ..., "options": ["default", ...], "default": [...]}
        // could never branch on "default" - reject the option value up front
        let def = PopupDefinition::new(
            "Test",
            vec![Element::Select {
                select: "Config".to_string(),
                id: "config".to_string(),
                options: vec![
                    OptionValue::Simple("default".to_string()),
                    OptionValue::Simple("custom".to_string()),
                ],
                default: None,
                style: None,
                option_children: HashMap::new(),
                reveals: vec![],
                when: None,
            }],
        );
        let issues = validate_definition(&def);
        assert_eq!(issues.len(), 1);
        assert!(issues[0]
            .message
            .contains("collides with the \"default\" field name"));
    }

    #[test]
    fn test_multi_default_must_be_options() {
        let def = PopupDefinition::new(
            "Test",
            vec![Element::Multi {
                multi: "Features".to_string(),
                id: "features".to_string(),
                options: vec![
                    OptionValue::Simple("Alpha".to_string()),
                    OptionValue::Simple("Beta".to_string()),
                ],
                default: vec!["Alpha".to_string(), "Betaa".to_string()],
                option_children: HashMap::new(),
                reveals: vec![],
                when: None,
            }],
        );
        let issues = validate_definition(&def);
        assert_eq!(issues.len(), 1);
        assert!(issues[0]
            .message
            .contains("\"Betaa\" is not one of the options"));
        assert!(issues[0].message.contains("did you mean \"Beta\""));
    }

    fn page(label: &str, elements: Vec<Element>) -> Element {
        Element::Page {
            page: label.to_string(),
            id: None,
            elements,
            when: None,
            next: vec![],
        }
    }

    #[test]
    fn test_edge_targets_must_be_page_ids() {
        let mut start = page("Start", vec![slider("a", 0.0, 10.0, None)]);
        if let Element::Page { next, .. } = &mut start {
            next.push(crate::Edge {
                to: "detials".into(),
                r#if: Some("anything".into()),
                when: None,
            });
            next.push(crate::Edge {
                to: "submit".into(),
                r#if: Some("done".into()),
                when: None,
            });
        }
        let def = PopupDefinition::new("T", vec![start, page("Details", vec![])]);
        let issues = validate_definition(&def);
        assert_eq!(issues.len(), 1, "{:?}", issues);
        assert!(issues[0].message.contains("did you mean \"details\""));
    }

    #[test]
    fn test_pages_must_not_mix_with_loose_elements() {
        let def = PopupDefinition::new(
            "Test",
            vec![
                page("Step 1", vec![slider("a", 0.0, 10.0, None)]),
                slider("loose", 0.0, 10.0, None),
            ],
        );
        let issues = validate_definition(&def);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("ONLY pages at the top level"));
    }

    #[test]
    fn test_nested_pages_rejected() {
        let def = PopupDefinition::new(
            "Test",
            vec![page(
                "Step 1",
                vec![page("Nested", vec![slider("a", 0.0, 10.0, None)])],
            )],
        );
        let issues = validate_definition(&def);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("only appear at the top level"));
    }

    #[test]
    fn test_valid_wizard_passes() {
        let def = PopupDefinition::new(
            "Test",
            vec![
                page("Step 1", vec![slider("a", 0.0, 10.0, None)]),
                page("Step 2", vec![slider("b", 0.0, 10.0, None)]),
            ],
        );
        assert_eq!(validate_definition(&def), vec![]);
    }

    #[test]
    fn test_unknown_top_level_field_is_error() {
        // {"button": "Deploy"} (meant "submit") must not be silently ignored
        let err = serde_json::from_str::<PopupDefinition>(
            r#"{"title": "T", "elements": [], "button": "Deploy"}"#,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("unknown field `button`"), "got: {}", err);
        assert!(
            err.contains("submit"),
            "expected-fields list missing: {}",
            err
        );
    }
}
