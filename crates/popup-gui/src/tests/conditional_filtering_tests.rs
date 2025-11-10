#[cfg(test)]
mod tests {
    use crate::json_parser::parse_popup_json_value;
    use popup_common::PopupState;
    use serde_json::json;

    #[test]
    fn test_simple_when_clause_filtering() {
        let json = json!({
            "title": "Test When Clause",
            "elements": [
                {
                    "checkbox": "Show Advanced",
                    "id": "show_advanced",
                    "default": false
                },
                {
                    "slider": "Advanced Setting",
                    "id": "advanced_setting",
                    "min": 0,
                    "max": 100,
                    "default": 50,
                    "when": "@show_advanced"
                }
            ]
        });

        let popup = parse_popup_json_value(json).unwrap();
        let mut state = PopupState::new(&popup);

        // Initially, checkbox is false, so when clause element should not appear
        state.button_clicked = Some("submit".to_string());

        // Collect active elements
        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );

        // Only the checkbox should be active
        assert_eq!(active_ids, vec!["show_advanced"]);

        // Now enable the checkbox
        *state.get_boolean_mut("show_advanced").unwrap() = true;

        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );

        // Now both checkbox and slider should be active
        assert!(active_ids.contains(&"show_advanced".to_string()));
        assert!(active_ids.contains(&"advanced_setting".to_string()));
        assert_eq!(active_ids.len(), 2);
    }

    #[test]
    fn test_nested_when_clause_filtering() {
        let json = json!({
            "title": "Nested When Clauses",
            "elements": [
                {
                    "checkbox": "Enable Features",
                    "id": "enable_features",
                    "default": false
                },
                {
                    "checkbox": "Advanced Mode",
                    "id": "advanced_mode",
                    "default": false,
                    "when": "@enable_features"
                },
                {
                    "slider": "Advanced Level",
                    "id": "advanced_level",
                    "min": 0,
                    "max": 10,
                    "when": "@advanced_mode"
                }
            ]
        });

        let popup = parse_popup_json_value(json).unwrap();
        let mut state = PopupState::new(&popup);

        // Nothing enabled
        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert_eq!(active_ids, vec!["enable_features"]);

        // Enable first level
        *state.get_boolean_mut("enable_features").unwrap() = true;

        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert!(active_ids.contains(&"enable_features".to_string()));
        assert!(active_ids.contains(&"advanced_mode".to_string()));
        assert!(!active_ids.contains(&"advanced_level".to_string()));

        // Enable second level
        *state.get_boolean_mut("advanced_mode").unwrap() = true;

        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert!(active_ids.contains(&"enable_features".to_string()));
        assert!(active_ids.contains(&"advanced_mode".to_string()));
        assert!(active_ids.contains(&"advanced_level".to_string()));
    }

    #[test]
    fn test_result_filtering_matches_active_elements() {
        let json = json!({
            "title": "Result Test",
            "elements": [
                {
                    "checkbox": "Show Options",
                    "id": "show_options",
                    "default": false
                },
                {
                    "slider": "Always Visible",
                    "id": "always_visible",
                    "min": 0,
                    "max": 100,
                    "default": 25
                },
                {
                    "slider": "Hidden Slider",
                    "id": "hidden_slider",
                    "min": 0,
                    "max": 100,
                    "default": 75,
                    "when": "@show_options"
                }
            ]
        });

        let popup = parse_popup_json_value(json).unwrap();
        let mut state = PopupState::new(&popup);
        state.button_clicked = Some("submit".to_string());

        // Get active IDs
        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );

        // Create result with active filtering
        let result = popup_common::PopupResult::from_state_with_active_elements(
            &state,
            &popup,
            &active_ids,
        );

        // Should only have the visible elements
        let values = match result {
            popup_common::PopupResult::Completed { values, .. } => values,
            _ => panic!("Expected Completed result"),
        };
        assert!(values.contains_key("show_options"));
        assert!(values.contains_key("always_visible"));
        assert!(!values.contains_key("hidden_slider"));

        // Enable the checkbox
        *state.get_boolean_mut("show_options").unwrap() = true;

        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );

        let result = popup_common::PopupResult::from_state_with_active_elements(
            &state,
            &popup,
            &active_ids,
        );

        // Now should include the conditional slider
        let values = match result {
            popup_common::PopupResult::Completed { values, .. } => values,
            _ => panic!("Expected Completed result"),
        };
        assert!(values.contains_key("show_options"));
        assert!(values.contains_key("always_visible"));
        assert!(values.contains_key("hidden_slider"));
    }

    #[test]
    fn test_multiselect_count_when_clause_filtering() {
        let json = json!({
            "title": "Count-based When Clause",
            "elements": [
                {
                    "multiselect": "Features",
                    "id": "features",
                    "options": ["Feature A", "Feature B", "Feature C", "Feature D"]
                },
                {
                    "textbox": "Premium Config",
                    "id": "premium_config",
                    "placeholder": "Available with 3+ features",
                    "when": "count(@features) > 2"
                }
            ]
        });

        let popup = parse_popup_json_value(json).unwrap();
        let mut state = PopupState::new(&popup);

        // No selections
        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert_eq!(active_ids, vec!["features"]);

        // Select 2 features (not enough)
        {
            let selections = state.get_multichoice_mut("features").unwrap();
            selections[0] = true;
            selections[1] = true;
        }

        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert_eq!(active_ids, vec!["features"]);

        // Select 3 features (enough)
        {
            let selections = state.get_multichoice_mut("features").unwrap();
            selections[2] = true;
        }

        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert!(active_ids.contains(&"features".to_string()));
        assert!(active_ids.contains(&"premium_config".to_string()));
    }

    #[test]
    fn test_when_clause_within_group_filtering() {
        let json = json!({
            "title": "Group with When Clause",
            "elements": [
                {
                    "group": "Settings Group",
                    "id": "settings_group",
                    "elements": [
                        {
                            "checkbox": "Group Option",
                            "id": "group_option",
                            "default": false
                        },
                        {
                            "slider": "Group Slider",
                            "id": "group_slider",
                            "min": 0,
                            "max": 100,
                            "when": "@group_option"
                        }
                    ]
                }
            ]
        });

        let popup = parse_popup_json_value(json).unwrap();
        let mut state = PopupState::new(&popup);

        // Initially only checkbox visible
        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert_eq!(active_ids, vec!["group_option"]);

        // Enable checkbox to show when clause slider
        *state.get_boolean_mut("group_option").unwrap() = true;

        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert!(active_ids.contains(&"group_option".to_string()));
        assert!(active_ids.contains(&"group_slider".to_string()));
    }

    #[test]
    fn test_complex_comparison_operators() {
        let json = json!({
            "title": "Complex Comparisons",
            "elements": [
                {
                    "multiselect": "Items",
                    "id": "items",
                    "options": ["A", "B", "C", "D", "E"]
                },
                {
                    "textbox": "Bulk Config",
                    "id": "bulk_config",
                    "when": "count(@items) >= 3"
                },
                {
                    "textbox": "Single Config",
                    "id": "single_config",
                    "when": "count(@items) == 1"
                },
                {
                    "checkbox": "Simple Mode",
                    "id": "simple_mode",
                    "when": "count(@items) < 2"
                }
            ]
        });

        let popup = parse_popup_json_value(json).unwrap();
        let mut state = PopupState::new(&popup);

        // No selections - count = 0, should trigger "< 2" condition
        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert!(active_ids.contains(&"items".to_string()));
        assert!(!active_ids.contains(&"bulk_config".to_string()));
        assert!(!active_ids.contains(&"single_config".to_string()));
        assert!(active_ids.contains(&"simple_mode".to_string()));

        // Select exactly 1 - should trigger "== 1" and "< 2" conditions
        {
            let selections = state.get_multichoice_mut("items").unwrap();
            selections[0] = true;
        }

        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert!(active_ids.contains(&"items".to_string()));
        assert!(!active_ids.contains(&"bulk_config".to_string()));
        assert!(active_ids.contains(&"single_config".to_string()));
        assert!(active_ids.contains(&"simple_mode".to_string()));

        // Select exactly 3 - should trigger ">= 3" condition only
        {
            let selections = state.get_multichoice_mut("items").unwrap();
            selections[1] = true;
            selections[2] = true;
        }

        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert!(active_ids.contains(&"items".to_string()));
        assert!(active_ids.contains(&"bulk_config".to_string()));
        assert!(!active_ids.contains(&"single_config".to_string()));
        assert!(!active_ids.contains(&"simple_mode".to_string()));
    }

    #[test]
    fn test_comprehensive_result_comparison() {
        // Test that results from old method include inactive elements
        // while new method excludes them
        let json = json!({
            "title": "Result Comparison",
            "elements": [
                {
                    "checkbox": "Enable Advanced",
                    "id": "enable_advanced",
                    "default": false
                },
                {
                    "slider": "Basic Setting",
                    "id": "basic_setting",
                    "min": 0,
                    "max": 100,
                    "default": 30
                },
                {
                    "slider": "Advanced Setting",
                    "id": "advanced_setting",
                    "min": 0,
                    "max": 100,
                    "default": 80,
                    "when": "@enable_advanced"
                },
                {
                    "textbox": "Advanced Config",
                    "id": "advanced_config",
                    "placeholder": "Enter config",
                    "when": "@enable_advanced"
                }
            ]
        });

        let popup = parse_popup_json_value(json).unwrap();
        let mut state = PopupState::new(&popup);
        state.button_clicked = Some("submit".to_string());

        // Compare old method (includes all) vs new method (filters inactive)
        let old_result = popup_common::PopupResult::from_state_with_context(&state, &popup);

        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        let new_result = popup_common::PopupResult::from_state_with_active_elements(
            &state,
            &popup,
            &active_ids,
        );

        // Destructure results to get values
        let old_values = match old_result {
            popup_common::PopupResult::Completed { values, .. } => values,
            _ => panic!("Expected Completed result"),
        };
        let new_values = match new_result {
            popup_common::PopupResult::Completed { values, .. } => values,
            _ => panic!("Expected Completed result"),
        };

        // Old method includes phantom values from inactive when clause elements
        assert!(old_values.contains_key("enable_advanced"));
        assert!(old_values.contains_key("basic_setting"));
        assert!(old_values.contains_key("advanced_setting")); // Phantom!
        assert!(!old_values.contains_key("advanced_config")); // Empty text skipped

        // New method excludes inactive when clause values
        assert!(new_values.contains_key("enable_advanced"));
        assert!(new_values.contains_key("basic_setting"));
        assert!(!new_values.contains_key("advanced_setting")); // Correctly filtered!
        assert!(!new_values.contains_key("advanced_config"));

        // Verify values match for active elements
        assert_eq!(
            old_values.get("enable_advanced"),
            new_values.get("enable_advanced")
        );
        assert_eq!(
            old_values.get("basic_setting"),
            new_values.get("basic_setting")
        );
    }

    #[test]
    fn test_checkbox_reveals_field() {
        let json = json!({
            "title": "Checkbox Reveals",
            "elements": [
                {
                    "checkbox": "Enable Features",
                    "id": "enable_features",
                    "default": false,
                    "reveals": [
                        {
                            "slider": "Feature Level",
                            "id": "feature_level",
                            "min": 0,
                            "max": 10,
                            "default": 5
                        }
                    ]
                }
            ]
        });

        let popup = parse_popup_json_value(json).unwrap();
        let mut state = PopupState::new(&popup);

        // Initially checkbox is unchecked, reveals should not appear
        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert_eq!(active_ids, vec!["enable_features"]);

        // Check the checkbox
        *state.get_boolean_mut("enable_features").unwrap() = true;

        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );

        // Both checkbox and slider should be active
        assert!(active_ids.contains(&"enable_features".to_string()));
        assert!(active_ids.contains(&"feature_level".to_string()));
        assert_eq!(active_ids.len(), 2);
    }

    #[test]
    fn test_choice_option_children() {
        let json = json!({
            "title": "Choice Option Children",
            "elements": [
                {
                    "choice": "Mode",
                    "id": "mode",
                    "options": ["Simple", "Advanced"],
                    "default": 0,
                    "Advanced": [
                        {
                            "slider": "Complexity",
                            "id": "complexity",
                            "min": 1,
                            "max": 10,
                            "default": 5
                        }
                    ]
                }
            ]
        });

        let popup = parse_popup_json_value(json).unwrap();
        let mut state = PopupState::new(&popup);

        // Initially "Simple" is selected (index 0), no children
        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert_eq!(active_ids, vec!["mode"]);

        // Select "Advanced" (index 1) which has children
        *state.get_choice_mut("mode").unwrap() = Some(1);

        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );

        // Both choice and child slider should be active
        assert!(active_ids.contains(&"mode".to_string()));
        assert!(active_ids.contains(&"complexity".to_string()));
        assert_eq!(active_ids.len(), 2);

        // Switch back to "Simple" (index 0)
        *state.get_choice_mut("mode").unwrap() = Some(0);

        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );

        // Only choice should be active
        assert_eq!(active_ids, vec!["mode"]);
    }

    #[test]
    fn test_multiselect_option_children() {
        let json = json!({
            "title": "Multiselect Option Children",
            "elements": [
                {
                    "multiselect": "Features",
                    "id": "features",
                    "options": ["Basic", "Advanced", "Expert"],
                    "Advanced": [
                        {
                            "slider": "Advanced Level",
                            "id": "advanced_level",
                            "min": 1,
                            "max": 5,
                            "default": 3
                        }
                    ],
                    "Expert": [
                        {
                            "textbox": "Expert Config",
                            "id": "expert_config",
                            "placeholder": "Enter config"
                        }
                    ]
                }
            ]
        });

        let popup = parse_popup_json_value(json).unwrap();
        let mut state = PopupState::new(&popup);

        // Initially nothing is selected
        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert_eq!(active_ids, vec!["features"]);

        // Select "Basic" (index 0) - no children
        state.get_multichoice_mut("features").unwrap()[0] = true;

        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert_eq!(active_ids, vec!["features"]);

        // Select "Advanced" (index 1) - has children
        state.get_multichoice_mut("features").unwrap()[1] = true;

        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert!(active_ids.contains(&"features".to_string()));
        assert!(active_ids.contains(&"advanced_level".to_string()));
        assert_eq!(active_ids.len(), 2);

        // Also select "Expert" (index 2) - has different children
        state.get_multichoice_mut("features").unwrap()[2] = true;

        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert!(active_ids.contains(&"features".to_string()));
        assert!(active_ids.contains(&"advanced_level".to_string()));
        assert!(active_ids.contains(&"expert_config".to_string()));
        assert_eq!(active_ids.len(), 3);

        // Deselect "Advanced" (index 1)
        state.get_multichoice_mut("features").unwrap()[1] = false;

        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert!(active_ids.contains(&"features".to_string()));
        assert!(!active_ids.contains(&"advanced_level".to_string()));
        assert!(active_ids.contains(&"expert_config".to_string()));
        assert_eq!(active_ids.len(), 2);
    }

    #[test]
    fn test_slider_reveals_with_when_clause() {
        // Test case from user bug report: slider with reveals containing when clause
        let json = json!({
            "title": "Slider Reveals Test",
            "elements": [
                {
                    "slider": "How deep should we go?",
                    "id": "depth",
                    "min": 1,
                    "max": 5,
                    "default": 1,
                    "reveals": [
                        {
                            "text": "✓ Slider reveal working",
                            "id": "reveal_text",
                            "when": "@depth >= 3"
                        }
                    ]
                }
            ]
        });

        let popup = parse_popup_json_value(json).unwrap();
        let mut state = PopupState::new(&popup);

        // Initially depth is 1, so when clause should not be satisfied
        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert_eq!(active_ids, vec!["depth"]);
        assert!(!active_ids.contains(&"reveal_text".to_string()));

        // Set depth to 2 - still below threshold
        *state.get_number_mut("depth").unwrap() = 2.0;
        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert_eq!(active_ids, vec!["depth"]);
        assert!(!active_ids.contains(&"reveal_text".to_string()));

        // Set depth to 3 - exactly at threshold, should appear
        *state.get_number_mut("depth").unwrap() = 3.0;
        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert!(active_ids.contains(&"depth".to_string()));
        assert!(active_ids.contains(&"reveal_text".to_string()));
        assert_eq!(active_ids.len(), 2);

        // Set depth to 5 - well above threshold, should still appear
        *state.get_number_mut("depth").unwrap() = 5.0;
        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert!(active_ids.contains(&"depth".to_string()));
        assert!(active_ids.contains(&"reveal_text".to_string()));
        assert_eq!(active_ids.len(), 2);
    }

    #[test]
    fn test_textbox_no_reveals_field() {
        // Textbox should not have reveals field (protocol consistency)
        let json = json!({
            "title": "Textbox Test",
            "elements": [
                {
                    "textbox": "Enter configuration",
                    "id": "config",
                    "placeholder": "JSON config..."
                }
            ]
        });

        let popup = parse_popup_json_value(json).unwrap();
        let state = PopupState::new(&popup);

        // Only textbox should be active
        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert_eq!(active_ids, vec!["config"]);
    }

    #[test]
    fn test_multiselect_reveals_conditional_on_selection() {
        // Test multiselect reveals only appear when ANY option is selected
        let json = json!({
            "title": "Multiselect Reveals Test",
            "elements": [
                {
                    "multiselect": "Select features",
                    "id": "features",
                    "options": ["Feature A", "Feature B", "Feature C"],
                    "reveals": [
                        {
                            "text": "✓ At least one feature selected",
                            "id": "selection_notice"
                        }
                    ]
                }
            ]
        });

        let popup = parse_popup_json_value(json).unwrap();
        let mut state = PopupState::new(&popup);

        // Initially no selections, reveals should NOT appear
        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert_eq!(active_ids, vec!["features"]);
        assert!(!active_ids.contains(&"selection_notice".to_string()));

        // Select one option - reveals should NOW appear
        state.get_multichoice_mut("features").unwrap()[0] = true;
        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert!(active_ids.contains(&"features".to_string()));
        assert!(active_ids.contains(&"selection_notice".to_string()));
        assert_eq!(active_ids.len(), 2);

        // Deselect all - reveals should disappear
        state.get_multichoice_mut("features").unwrap()[0] = false;
        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert_eq!(active_ids, vec!["features"]);
        assert!(!active_ids.contains(&"selection_notice".to_string()));

        // Select multiple - reveals should still appear
        state.get_multichoice_mut("features").unwrap()[1] = true;
        state.get_multichoice_mut("features").unwrap()[2] = true;
        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert!(active_ids.contains(&"features".to_string()));
        assert!(active_ids.contains(&"selection_notice".to_string()));
        assert_eq!(active_ids.len(), 2);
    }

    #[test]
    fn test_choice_reveals_conditional_on_selection() {
        // Test choice reveals only appear when ANY option is selected
        let json = json!({
            "title": "Choice Reveals Test",
            "elements": [
                {
                    "choice": "Select mode",
                    "id": "mode",
                    "options": ["Basic", "Advanced", "Expert"],
                    "reveals": [
                        {
                            "text": "Mode selected",
                            "id": "selection_notice"
                        }
                    ]
                }
            ]
        });

        let popup = parse_popup_json_value(json).unwrap();
        let mut state = PopupState::new(&popup);

        // Initially no selection, reveals should NOT appear
        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert_eq!(active_ids, vec!["mode"]);
        assert!(!active_ids.contains(&"selection_notice".to_string()));

        // Select "Basic" (index 0) - reveals should NOW appear
        *state.get_choice_mut("mode").unwrap() = Some(0);
        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert!(active_ids.contains(&"mode".to_string()));
        assert!(active_ids.contains(&"selection_notice".to_string()));
        assert_eq!(active_ids.len(), 2);

        // Clear selection - reveals should disappear
        *state.get_choice_mut("mode").unwrap() = None;
        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert_eq!(active_ids, vec!["mode"]);
        assert!(!active_ids.contains(&"selection_notice".to_string()));

        // Select different option - reveals should appear again
        *state.get_choice_mut("mode").unwrap() = Some(2);
        let active_ids = crate::gui::tests::collect_active_elements_for_test(
            &popup.elements,
            &state,
            &popup.elements,
        );
        assert!(active_ids.contains(&"mode".to_string()));
        assert!(active_ids.contains(&"selection_notice".to_string()));
        assert_eq!(active_ids.len(), 2);
    }
}
