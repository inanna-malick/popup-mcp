#[cfg(test)]
mod tests {
    use crate::{
        json_parser::parse_popup_json_value,
        models::PopupState,
    };
    use serde_json::json;

    #[test]
    fn test_simple_conditional_filtering() {
        let json = json!({
            "title": "Test Conditional",
            "elements": [
                {
                    "type": "checkbox",
                    "label": "Show Advanced",
                    "default": false
                },
                {
                    "type": "conditional",
                    "condition": "Show Advanced",
                    "elements": [
                        {
                            "type": "slider",
                            "label": "Advanced Setting",
                            "min": 0,
                            "max": 100,
                            "default": 50
                        }
                    ]
                }
            ]
        });

        let popup = parse_popup_json_value(json).unwrap();
        let mut state = PopupState::new(&popup);

        // Initially, checkbox is false, so conditional content should not appear
        state.button_clicked = Some("submit".to_string());

        // Collect active elements
        let active_labels = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );

        // Only the checkbox should be active
        assert_eq!(active_labels, vec!["Show Advanced"]);

        // Now enable the checkbox
        *state.get_boolean_mut("Show Advanced").unwrap() = true;

        let active_labels = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );

        // Now both checkbox and slider should be active
        assert!(active_labels.contains(&"Show Advanced".to_string()));
        assert!(active_labels.contains(&"Advanced Setting".to_string()));
        assert_eq!(active_labels.len(), 2);
    }


    #[test]
    fn test_nested_conditional_filtering() {
        let json = json!({
            "title": "Nested Conditionals",
            "elements": [
                {
                    "type": "checkbox",
                    "label": "Enable Features",
                    "default": false
                },
                {
                    "type": "conditional",
                    "condition": "Enable Features",
                    "elements": [
                        {
                            "type": "checkbox",
                            "label": "Advanced Mode",
                            "default": false
                        },
                        {
                            "type": "conditional",
                            "condition": "Advanced Mode",
                            "elements": [
                                {
                                    "type": "slider",
                                    "label": "Advanced Level",
                                    "min": 0,
                                    "max": 10
                                }
                            ]
                        }
                    ]
                }
            ]
        });

        let popup = parse_popup_json_value(json).unwrap();
        let mut state = PopupState::new(&popup);

        // Nothing enabled
        let active_labels = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert_eq!(active_labels, vec!["Enable Features"]);

        // Enable first level
        *state.get_boolean_mut("Enable Features").unwrap() = true;

        let active_labels = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert!(active_labels.contains(&"Enable Features".to_string()));
        assert!(active_labels.contains(&"Advanced Mode".to_string()));
        assert!(!active_labels.contains(&"Advanced Level".to_string()));

        // Enable second level
        *state.get_boolean_mut("Advanced Mode").unwrap() = true;

        let active_labels = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert!(active_labels.contains(&"Enable Features".to_string()));
        assert!(active_labels.contains(&"Advanced Mode".to_string()));
        assert!(active_labels.contains(&"Advanced Level".to_string()));
    }

    #[test]
    fn test_result_filtering_matches_active_elements() {
        let json = json!({
            "title": "Result Test",
            "elements": [
                {
                    "type": "checkbox",
                    "label": "Show Options",
                    "default": false
                },
                {
                    "type": "slider",
                    "label": "Always Visible",
                    "min": 0,
                    "max": 100,
                    "default": 25
                },
                {
                    "type": "conditional",
                    "condition": "Show Options",
                    "elements": [
                        {
                            "type": "slider",
                            "label": "Hidden Slider",
                            "min": 0,
                            "max": 100,
                            "default": 75
                        }
                    ]
                }
            ]
        });

        let popup = parse_popup_json_value(json).unwrap();
        let mut state = PopupState::new(&popup);
        state.button_clicked = Some("submit".to_string());

        // Get active labels
        let active_labels = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );

        // Create result with active filtering
        let result = crate::models::PopupResult::from_state_with_active_elements(
            &state,
            &popup,
            &active_labels,
        );

        // Should only have the visible elements
        assert!(result.values.contains_key("Show Options"));
        assert!(result.values.contains_key("Always Visible"));
        assert!(!result.values.contains_key("Hidden Slider"));

        // Enable the checkbox
        *state.get_boolean_mut("Show Options").unwrap() = true;

        let active_labels = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );

        let result = crate::models::PopupResult::from_state_with_active_elements(
            &state,
            &popup,
            &active_labels,
        );

        // Now should include the conditional slider
        assert!(result.values.contains_key("Show Options"));
        assert!(result.values.contains_key("Always Visible"));
        assert!(result.values.contains_key("Hidden Slider"));
    }

    #[test]
    fn test_multiselect_count_conditional_filtering() {
        let json = json!({
            "title": "Count-based Conditional",
            "elements": [
                {
                    "type": "multiselect",
                    "label": "Features",
                    "options": ["Feature A", "Feature B", "Feature C", "Feature D"]
                },
                {
                    "type": "conditional",
                    "condition": {
                        "field": "Features",
                        "count": ">2"
                    },
                    "elements": [
                        {
                            "type": "textbox",
                            "label": "Premium Config",
                            "placeholder": "Available with 3+ features"
                        }
                    ]
                }
            ]
        });

        let popup = parse_popup_json_value(json).unwrap();
        let mut state = PopupState::new(&popup);

        // No selections
        let active_labels = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert_eq!(active_labels, vec!["Features"]);

        // Select 2 features (not enough)
        {
            let selections = state.get_multichoice_mut("Features").unwrap();
            selections[0] = true;
            selections[1] = true;
        }

        let active_labels = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert_eq!(active_labels, vec!["Features"]);

        // Select 3 features (enough)
        {
            let selections = state.get_multichoice_mut("Features").unwrap();
            selections[2] = true;
        }

        let active_labels = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert!(active_labels.contains(&"Features".to_string()));
        assert!(active_labels.contains(&"Premium Config".to_string()));
    }

    #[test]
    fn test_conditional_within_group_filtering() {
        let json = json!({
            "title": "Group with Conditional",
            "elements": [
                {
                    "type": "group",
                    "label": "Settings Group",
                    "elements": [
                        {
                            "type": "checkbox",
                            "label": "Group Option",
                            "default": false
                        },
                        {
                            "type": "conditional",
                            "condition": "Group Option",
                            "elements": [
                                {
                                    "type": "slider",
                                    "label": "Group Slider",
                                    "min": 0,
                                    "max": 100
                                }
                            ]
                        }
                    ]
                }
            ]
        });

        let popup = parse_popup_json_value(json).unwrap();
        let mut state = PopupState::new(&popup);

        // Initially only checkbox visible
        let active_labels = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert_eq!(active_labels, vec!["Group Option"]);

        // Enable checkbox to show conditional slider
        *state.get_boolean_mut("Group Option").unwrap() = true;

        let active_labels = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert!(active_labels.contains(&"Group Option".to_string()));
        assert!(active_labels.contains(&"Group Slider".to_string()));
    }


    #[test]
    fn test_complex_comparison_operators() {
        let json = json!({
            "title": "Complex Comparisons",
            "elements": [
                {
                    "type": "multiselect",
                    "label": "Items",
                    "options": ["A", "B", "C", "D", "E"]
                },
                {
                    "type": "conditional",
                    "condition": {
                        "field": "Items",
                        "count": ">=3"
                    },
                    "elements": [
                        {
                            "type": "textbox",
                            "label": "Bulk Config"
                        }
                    ]
                },
                {
                    "type": "conditional",
                    "condition": {
                        "field": "Items",
                        "count": "=1"
                    },
                    "elements": [
                        {
                            "type": "textbox",
                            "label": "Single Config"
                        }
                    ]
                },
                {
                    "type": "conditional",
                    "condition": {
                        "field": "Items",
                        "count": "<2"
                    },
                    "elements": [
                        {
                            "type": "checkbox",
                            "label": "Simple Mode"
                        }
                    ]
                }
            ]
        });

        let popup = parse_popup_json_value(json).unwrap();
        let mut state = PopupState::new(&popup);

        // No selections - count = 0, should trigger "< 2" condition
        let active_labels = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert!(active_labels.contains(&"Items".to_string()));
        assert!(!active_labels.contains(&"Bulk Config".to_string()));
        assert!(!active_labels.contains(&"Single Config".to_string()));
        assert!(active_labels.contains(&"Simple Mode".to_string()));

        // Select exactly 1 - should trigger "= 1" and "< 2" conditions
        {
            let selections = state.get_multichoice_mut("Items").unwrap();
            selections[0] = true;
        }

        let active_labels = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert!(active_labels.contains(&"Items".to_string()));
        assert!(!active_labels.contains(&"Bulk Config".to_string()));
        assert!(active_labels.contains(&"Single Config".to_string()));
        assert!(active_labels.contains(&"Simple Mode".to_string()));

        // Select exactly 3 - should trigger ">= 3" condition only
        {
            let selections = state.get_multichoice_mut("Items").unwrap();
            selections[1] = true;
            selections[2] = true;
        }

        let active_labels = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert!(active_labels.contains(&"Items".to_string()));
        assert!(active_labels.contains(&"Bulk Config".to_string()));
        assert!(!active_labels.contains(&"Single Config".to_string()));
        assert!(!active_labels.contains(&"Simple Mode".to_string()));
    }

    #[test]
    fn test_comprehensive_result_comparison() {
        // Test that results from old method include inactive elements
        // while new method excludes them
        let json = json!({
            "title": "Result Comparison",
            "elements": [
                {
                    "type": "checkbox",
                    "label": "Enable Advanced",
                    "default": false
                },
                {
                    "type": "slider",
                    "label": "Basic Setting",
                    "min": 0,
                    "max": 100,
                    "default": 30
                },
                {
                    "type": "conditional",
                    "condition": "Enable Advanced",
                    "elements": [
                        {
                            "type": "slider",
                            "label": "Advanced Setting",
                            "min": 0,
                            "max": 100,
                            "default": 80
                        },
                        {
                            "type": "textbox",
                            "label": "Advanced Config",
                            "placeholder": "Enter config"
                        }
                    ]
                }
            ]
        });

        let popup = parse_popup_json_value(json).unwrap();
        let mut state = PopupState::new(&popup);
        state.button_clicked = Some("submit".to_string());

        // Compare old method (includes all) vs new method (filters inactive)
        let old_result = crate::models::PopupResult::from_state_with_context(&state, &popup);

        let active_labels = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        let new_result = crate::models::PopupResult::from_state_with_active_elements(
            &state,
            &popup,
            &active_labels,
        );

        // Old method includes phantom values from inactive conditional
        assert!(old_result.values.contains_key("Enable Advanced"));
        assert!(old_result.values.contains_key("Basic Setting"));
        assert!(old_result.values.contains_key("Advanced Setting")); // Phantom!
        assert!(!old_result.values.contains_key("Advanced Config")); // Empty text skipped

        // New method excludes inactive conditional values
        assert!(new_result.values.contains_key("Enable Advanced"));
        assert!(new_result.values.contains_key("Basic Setting"));
        assert!(!new_result.values.contains_key("Advanced Setting")); // Correctly filtered!
        assert!(!new_result.values.contains_key("Advanced Config"));

        // Verify values match for active elements
        assert_eq!(
            old_result.values.get("Enable Advanced"),
            new_result.values.get("Enable Advanced")
        );
        assert_eq!(
            old_result.values.get("Basic Setting"),
            new_result.values.get("Basic Setting")
        );
    }
}