use popup_common::{Element, OptionValue, PopupDefinition};
use std::collections::HashMap;

/// Recursively inject "Other" options into all Multi and Select elements
pub fn inject_other_options(mut def: PopupDefinition) -> PopupDefinition {
    inject_other_in_elements(&mut def.elements);
    def
}

/// Recursively process elements slice
fn inject_other_in_elements(elements: &mut [Element]) {
    for element in elements.iter_mut() {
        inject_other_in_element(element);
    }
}

/// Process a single element and recurse into children
fn inject_other_in_element(element: &mut Element) {
    match element {
        Element::Multi {
            options,
            option_children,
            reveals,
            id,
            ..
        }
        | Element::Select {
            options,
            option_children,
            reveals,
            id,
            ..
        } => {
            inject_other_option(options, option_children, id);
            // Recurse into children
            for children in option_children.values_mut() {
                inject_other_in_elements(children);
            }
            inject_other_in_elements(reveals);
        }
        Element::Check { reveals, .. } => {
            inject_other_in_elements(reveals);
        }
        Element::Group { elements, .. } => {
            inject_other_in_elements(elements);
        }
        _ => {} // No children to recurse into
    }
}

/// Ensure every select/multi has an "Other" escape hatch with an attached
/// text input, preserving the documented "<id>_other_text" result contract
/// even when the author supplied their own "Other" option
fn inject_other_option(
    options: &mut Vec<OptionValue>,
    option_children: &mut HashMap<String, Vec<Element>>,
    element_id: &str,
) {
    const OTHER_LABEL: &str = popup_common::validate::OTHER_LABEL;

    let is_other = |value: &str| {
        value.eq_ignore_ascii_case("other") || value.eq_ignore_ascii_case(OTHER_LABEL)
    };

    // Find an author-supplied "Other" (case-insensitive), else append ours
    let other_value = match options.iter().find(|opt| is_other(opt.value())) {
        Some(existing) => existing.value().to_string(),
        None => {
            options.push(OptionValue::Simple(OTHER_LABEL.to_string()));
            OTHER_LABEL.to_string()
        }
    };

    // Attach the free-text input unless the author already nested their own
    // children under that option
    option_children.entry(other_value).or_insert_with(|| {
        vec![Element::Input {
            input: "Please specify".to_string(),
            id: format!("{}_other_text", element_id),
            placeholder: None,
            default: None,
            rows: None,
            when: None,
        }]
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn multi(label: &str, id: &str, options: &[&str]) -> Element {
        Element::Multi {
            multi: label.to_string(),
            id: id.to_string(),
            options: options
                .iter()
                .map(|o| OptionValue::Simple(o.to_string()))
                .collect(),
            default: vec![],
            option_children: HashMap::new(),
            reveals: vec![],
            when: None,
        }
    }

    fn select_with_children(
        label: &str,
        id: &str,
        options: Vec<OptionValue>,
        option_children: HashMap<String, Vec<Element>>,
    ) -> Element {
        Element::Select {
            select: label.to_string(),
            id: id.to_string(),
            options,
            default: None,
            style: None,
            option_children,
            reveals: vec![],
            when: None,
        }
    }

    fn simple_options(options: &[&str]) -> Vec<OptionValue> {
        options
            .iter()
            .map(|o| OptionValue::Simple(o.to_string()))
            .collect()
    }

    #[test]
    fn test_inject_other_multi() {
        let def = PopupDefinition::new("Test", vec![multi("Features", "features", &["A", "B"])]);

        let transformed = inject_other_options(def);

        match &transformed.elements[0] {
            Element::Multi {
                options,
                option_children,
                ..
            } => {
                assert_eq!(options.len(), 3);
                assert_eq!(options[2].value(), "Other (please specify)");

                // Verify text input was added
                let other_children = option_children.get("Other (please specify)").unwrap();
                assert_eq!(other_children.len(), 1);

                match &other_children[0] {
                    Element::Input { id, input, .. } => {
                        assert_eq!(id, "features_other_text");
                        assert_eq!(input, "Please specify");
                    }
                    _ => panic!("Expected Input element"),
                }
            }
            _ => panic!("Expected Multi element"),
        }
    }

    #[test]
    fn test_inject_other_select() {
        let def = PopupDefinition::new(
            "Test",
            vec![select_with_children(
                "Mode",
                "mode",
                simple_options(&["X", "Y"]),
                HashMap::new(),
            )],
        );

        let transformed = inject_other_options(def);

        match &transformed.elements[0] {
            Element::Select {
                options,
                option_children,
                ..
            } => {
                assert_eq!(options.len(), 3);
                assert_eq!(options[2].value(), "Other (please specify)");

                // Verify text input was added
                let other_children = option_children.get("Other (please specify)").unwrap();
                assert_eq!(other_children.len(), 1);

                match &other_children[0] {
                    Element::Input { id, .. } => {
                        assert_eq!(id, "mode_other_text");
                    }
                    _ => panic!("Expected Input element"),
                }
            }
            _ => panic!("Expected Select element"),
        }
    }

    #[test]
    fn test_idempotent_other_exists_lowercase() {
        let def = PopupDefinition::new(
            "Test",
            vec![select_with_children(
                "Mode",
                "mode",
                simple_options(&["A", "other"]),
                HashMap::new(),
            )],
        );

        let transformed = inject_other_options(def);

        match &transformed.elements[0] {
            Element::Select {
                options,
                option_children,
                ..
            } => {
                // Should still be 2 options (not 3)...
                assert_eq!(options.len(), 2);
                // ...but the author-supplied "other" still gets the text
                // input, preserving the <id>_other_text result contract
                let children = option_children.get("other").unwrap();
                match &children[0] {
                    Element::Input { id, .. } => assert_eq!(id, "mode_other_text"),
                    _ => panic!("Expected Input element"),
                }
            }
            _ => panic!("Expected Select element"),
        }
    }

    #[test]
    fn test_idempotent_other_exists_with_description() {
        let def = PopupDefinition::new(
            "Test",
            vec![select_with_children(
                "Mode",
                "mode",
                vec![
                    OptionValue::Simple("A".to_string()),
                    OptionValue::WithDescription {
                        value: "Other".to_string(),
                        description: "Custom option".to_string(),
                    },
                ],
                HashMap::new(),
            )],
        );

        let transformed = inject_other_options(def);

        match &transformed.elements[0] {
            Element::Select { options, .. } => {
                // Should still be 2 options (not 3)
                assert_eq!(options.len(), 2);
            }
            _ => panic!("Expected Select element"),
        }
    }

    #[test]
    fn test_recursive_injection_in_reveals() {
        let def = PopupDefinition::new(
            "Test",
            vec![Element::Check {
                check: "Enable advanced".to_string(),
                id: "advanced".to_string(),
                default: false,
                reveals: vec![multi(
                    "Advanced features",
                    "advanced_features",
                    &["Feature1"],
                )],
                when: None,
            }],
        );

        let transformed = inject_other_options(def);

        match &transformed.elements[0] {
            Element::Check { reveals, .. } => match &reveals[0] {
                Element::Multi { options, .. } => {
                    assert_eq!(options.len(), 2);
                    assert_eq!(options[1].value(), "Other (please specify)");
                }
                _ => panic!("Expected Multi element in reveals"),
            },
            _ => panic!("Expected Check element"),
        }
    }

    #[test]
    fn test_recursive_injection_in_option_children() {
        let mut option_children = HashMap::new();
        option_children.insert(
            "Advanced".to_string(),
            vec![select_with_children(
                "Complexity",
                "complexity",
                simple_options(&["Low"]),
                HashMap::new(),
            )],
        );

        let def = PopupDefinition::new(
            "Test",
            vec![select_with_children(
                "Mode",
                "mode",
                simple_options(&["Simple", "Advanced"]),
                option_children,
            )],
        );

        let transformed = inject_other_options(def);

        match &transformed.elements[0] {
            Element::Select {
                options,
                option_children,
                ..
            } => {
                // Parent should have "Other" added
                assert_eq!(options.len(), 3);
                assert_eq!(options[2].value(), "Other (please specify)");

                // Child in option_children should also have "Other" added
                let advanced_children = option_children.get("Advanced").unwrap();
                match &advanced_children[0] {
                    Element::Select { options, .. } => {
                        assert_eq!(options.len(), 2);
                        assert_eq!(options[1].value(), "Other (please specify)");
                    }
                    _ => panic!("Expected Select element in option_children"),
                }
            }
            _ => panic!("Expected Select element"),
        }
    }

    #[test]
    fn test_recursive_injection_in_groups() {
        let def = PopupDefinition::new(
            "Test",
            vec![Element::Group {
                group: "Settings".to_string(),
                id: Some("settings".to_string()),
                elements: vec![multi("Options", "options", &["Opt1"])],
                when: None,
            }],
        );

        let transformed = inject_other_options(def);

        match &transformed.elements[0] {
            Element::Group { elements, .. } => match &elements[0] {
                Element::Multi { options, .. } => {
                    assert_eq!(options.len(), 2);
                    assert_eq!(options[1].value(), "Other (please specify)");
                }
                _ => panic!("Expected Multi element in group"),
            },
            _ => panic!("Expected Group element"),
        }
    }

    #[test]
    fn test_multi_level_nesting() {
        // Test deeply nested structure: Group > Check > reveals > Select > option_children > Multi
        let mut select_option_children = HashMap::new();
        select_option_children.insert(
            "Advanced".to_string(),
            vec![multi(
                "Advanced features",
                "advanced_features",
                &["Feature1"],
            )],
        );

        let def = PopupDefinition::new(
            "Test",
            vec![Element::Group {
                group: "Settings".to_string(),
                id: Some("settings".to_string()),
                elements: vec![Element::Check {
                    check: "Enable mode selector".to_string(),
                    id: "enable_mode".to_string(),
                    default: false,
                    reveals: vec![select_with_children(
                        "Mode",
                        "mode",
                        simple_options(&["Simple", "Advanced"]),
                        select_option_children,
                    )],
                    when: None,
                }],
                when: None,
            }],
        );

        let transformed = inject_other_options(def);

        // Navigate the nested structure to verify all levels got "Other" injected
        match &transformed.elements[0] {
            Element::Group { elements, .. } => match &elements[0] {
                Element::Check { reveals, .. } => match &reveals[0] {
                    Element::Select {
                        options,
                        option_children,
                        ..
                    } => {
                        // Select should have "Other"
                        assert_eq!(options.len(), 3);
                        assert_eq!(options[2].value(), "Other (please specify)");

                        // Multi inside Select's option_children should also have "Other"
                        let advanced_children = option_children.get("Advanced").unwrap();
                        match &advanced_children[0] {
                            Element::Multi { options, .. } => {
                                assert_eq!(options.len(), 2);
                                assert_eq!(options[1].value(), "Other (please specify)");
                            }
                            _ => panic!("Expected Multi in Select's option_children"),
                        }
                    }
                    _ => panic!("Expected Select in Check's reveals"),
                },
                _ => panic!("Expected Check in Group"),
            },
            _ => panic!("Expected Group element"),
        }
    }
}
