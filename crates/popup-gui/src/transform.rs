use popup_common::{Element, OptionValue, PopupDefinition};
use std::collections::HashMap;

/// Recursively inject "Other" options into all Multi and Select elements
pub fn inject_other_options(mut def: PopupDefinition) -> PopupDefinition {
    inject_other_in_elements(&mut def.elements);
    def
}

/// Recursively process elements vector
fn inject_other_in_elements(elements: &mut Vec<Element>) {
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
        } => {
            inject_other_option(options, option_children, id);
            // Recurse into children
            for children in option_children.values_mut() {
                inject_other_in_elements(children);
            }
            inject_other_in_elements(reveals);
        }
        Element::Select {
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

/// Add "Other" option if not already present (case-insensitive check)
fn inject_other_option(
    options: &mut Vec<OptionValue>,
    option_children: &mut HashMap<String, Vec<Element>>,
    element_id: &str,
) {
    const OTHER_LABEL: &str = "Other (please specify)";

    // Check if "Other" already exists (case-insensitive)
    let has_other = options.iter().any(|opt| {
        opt.value().eq_ignore_ascii_case("other")
            || opt.value().eq_ignore_ascii_case(OTHER_LABEL)
    });

    if !has_other {
        // Add "Other" to options
        options.push(OptionValue::Simple(OTHER_LABEL.to_string()));

        // Create text input element
        let text_input_id = format!("{}_other_text", element_id);
        let text_input = Element::Input {
            input: OTHER_LABEL.to_string(),
            id: text_input_id,
            placeholder: None,
            rows: None,
            when: None,
        };

        // Add to option_children (shown when "Other" is selected)
        option_children.insert(OTHER_LABEL.to_string(), vec![text_input]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject_other_multi() {
        let def = PopupDefinition {
            title: "Test".to_string(),
            elements: vec![Element::Multi {
                multi: "Features".to_string(),
                id: "features".to_string(),
                options: vec![
                    OptionValue::Simple("A".to_string()),
                    OptionValue::Simple("B".to_string()),
                ],
                option_children: HashMap::new(),
                reveals: vec![],
                when: None,
            }],
        };

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
                        assert_eq!(input, "Other (please specify)");
                    }
                    _ => panic!("Expected Input element"),
                }
            }
            _ => panic!("Expected Multi element"),
        }
    }

    #[test]
    fn test_inject_other_select() {
        let def = PopupDefinition {
            title: "Test".to_string(),
            elements: vec![Element::Select {
                select: "Mode".to_string(),
                id: "mode".to_string(),
                options: vec![
                    OptionValue::Simple("X".to_string()),
                    OptionValue::Simple("Y".to_string()),
                ],
                default: None,
                option_children: HashMap::new(),
                reveals: vec![],
                when: None,
            }],
        };

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
        let def = PopupDefinition {
            title: "Test".to_string(),
            elements: vec![Element::Select {
                select: "Mode".to_string(),
                id: "mode".to_string(),
                options: vec![
                    OptionValue::Simple("A".to_string()),
                    OptionValue::Simple("other".to_string()), // lowercase
                ],
                default: None,
                option_children: HashMap::new(),
                reveals: vec![],
                when: None,
            }],
        };

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
    fn test_idempotent_other_exists_with_description() {
        let def = PopupDefinition {
            title: "Test".to_string(),
            elements: vec![Element::Select {
                select: "Mode".to_string(),
                id: "mode".to_string(),
                options: vec![
                    OptionValue::Simple("A".to_string()),
                    OptionValue::WithDescription {
                        value: "Other".to_string(),
                        description: "Custom option".to_string(),
                    },
                ],
                default: None,
                option_children: HashMap::new(),
                reveals: vec![],
                when: None,
            }],
        };

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
        let def = PopupDefinition {
            title: "Test".to_string(),
            elements: vec![Element::Check {
                check: "Enable advanced".to_string(),
                id: "advanced".to_string(),
                default: false,
                reveals: vec![Element::Multi {
                    multi: "Advanced features".to_string(),
                    id: "advanced_features".to_string(),
                    options: vec![OptionValue::Simple("Feature1".to_string())],
                    option_children: HashMap::new(),
                    reveals: vec![],
                    when: None,
                }],
                when: None,
            }],
        };

        let transformed = inject_other_options(def);

        match &transformed.elements[0] {
            Element::Check { reveals, .. } => {
                match &reveals[0] {
                    Element::Multi { options, .. } => {
                        assert_eq!(options.len(), 2);
                        assert_eq!(options[1].value(), "Other (please specify)");
                    }
                    _ => panic!("Expected Multi element in reveals"),
                }
            }
            _ => panic!("Expected Check element"),
        }
    }

    #[test]
    fn test_recursive_injection_in_option_children() {
        let mut option_children = HashMap::new();
        option_children.insert(
            "Advanced".to_string(),
            vec![Element::Select {
                select: "Complexity".to_string(),
                id: "complexity".to_string(),
                options: vec![OptionValue::Simple("Low".to_string())],
                default: None,
                option_children: HashMap::new(),
                reveals: vec![],
                when: None,
            }],
        );

        let def = PopupDefinition {
            title: "Test".to_string(),
            elements: vec![Element::Select {
                select: "Mode".to_string(),
                id: "mode".to_string(),
                options: vec![
                    OptionValue::Simple("Simple".to_string()),
                    OptionValue::Simple("Advanced".to_string()),
                ],
                default: None,
                option_children,
                reveals: vec![],
                when: None,
            }],
        };

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
        let def = PopupDefinition {
            title: "Test".to_string(),
            elements: vec![Element::Group {
                group: "Settings".to_string(),
                id: Some("settings".to_string()),
                elements: vec![Element::Multi {
                    multi: "Options".to_string(),
                    id: "options".to_string(),
                    options: vec![OptionValue::Simple("Opt1".to_string())],
                    option_children: HashMap::new(),
                    reveals: vec![],
                    when: None,
                }],
                when: None,
            }],
        };

        let transformed = inject_other_options(def);

        match &transformed.elements[0] {
            Element::Group { elements, .. } => {
                match &elements[0] {
                    Element::Multi { options, .. } => {
                        assert_eq!(options.len(), 2);
                        assert_eq!(options[1].value(), "Other (please specify)");
                    }
                    _ => panic!("Expected Multi element in group"),
                }
            }
            _ => panic!("Expected Group element"),
        }
    }

    #[test]
    fn test_multi_level_nesting() {
        // Test deeply nested structure: Group > Check > reveals > Select > option_children > Multi
        let mut select_option_children = HashMap::new();
        select_option_children.insert(
            "Advanced".to_string(),
            vec![Element::Multi {
                multi: "Advanced features".to_string(),
                id: "advanced_features".to_string(),
                options: vec![OptionValue::Simple("Feature1".to_string())],
                option_children: HashMap::new(),
                reveals: vec![],
                when: None,
            }],
        );

        let def = PopupDefinition {
            title: "Test".to_string(),
            elements: vec![Element::Group {
                group: "Settings".to_string(),
                id: Some("settings".to_string()),
                elements: vec![Element::Check {
                    check: "Enable mode selector".to_string(),
                    id: "enable_mode".to_string(),
                    default: false,
                    reveals: vec![Element::Select {
                        select: "Mode".to_string(),
                        id: "mode".to_string(),
                        options: vec![
                            OptionValue::Simple("Simple".to_string()),
                            OptionValue::Simple("Advanced".to_string()),
                        ],
                        default: None,
                        option_children: select_option_children,
                        reveals: vec![],
                        when: None,
                    }],
                    when: None,
                }],
                when: None,
            }],
        };

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
