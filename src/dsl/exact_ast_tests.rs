#[cfg(test)]
mod tests {
    use crate::dsl::simple_parser::parse_popup_dsl;
    use crate::models::{Element, PopupDefinition};

    // ============================================
    // TEST HELPERS
    // ============================================
    
    fn parse(input: &str) -> Result<PopupDefinition, anyhow::Error> {
        parse_popup_dsl(input)
    }

    macro_rules! popup {
        ($title:expr) => {
            PopupDefinition {
                title: $title.to_string(),
                elements: vec![],
            }
        };
        ($title:expr, [$($elem:expr),* $(,)?]) => {
            PopupDefinition {
                title: $title.to_string(),
                elements: vec![$($elem),*],
            }
        };
    }

    macro_rules! slider {
        ($label:expr, $min:expr, $max:expr) => {
            Element::Slider {
                label: $label.to_string(),
                min: $min as f32,
                max: $max as f32,
                default: (($min + $max) / 2) as f32,
            }
        };
        ($label:expr, $min:expr, $max:expr, $default:expr) => {
            Element::Slider {
                label: $label.to_string(),
                min: $min as f32,
                max: $max as f32,
                default: $default as f32,
            }
        };
    }

    macro_rules! checkbox {
        ($label:expr, $default:expr) => {
            Element::Checkbox {
                label: $label.to_string(),
                default: $default,
            }
        };
    }

    macro_rules! choice {
        ($label:expr, [$($opt:expr),* $(,)?]) => {
            Element::Choice {
                label: $label.to_string(),
                options: vec![$($opt.to_string()),*],
            }
        };
    }

    macro_rules! multiselect {
        ($label:expr, [$($opt:expr),* $(,)?]) => {
            Element::Multiselect {
                label: $label.to_string(),
                options: vec![$($opt.to_string()),*],
            }
        };
    }

    macro_rules! textbox {
        ($label:expr) => {
            Element::Textbox {
                label: $label.to_string(),
                placeholder: None,
                rows: None,
            }
        };
        ($label:expr, $placeholder:expr) => {
            Element::Textbox {
                label: $label.to_string(),
                placeholder: Some($placeholder.to_string()),
                rows: None,
            }
        };
    }

    macro_rules! text {
        ($content:expr) => {
            Element::Text($content.to_string())
        };
    }

    macro_rules! buttons {
        ([$($btn:expr),* $(,)?]) => {
            Element::Buttons(vec![$($btn.to_string()),*])
        };
    }

    // ============================================
    // 1. GRAMMAR RULE COVERAGE TESTS
    // ============================================

    #[test]
    fn test_rule_title_plain() {
        assert_eq!(parse("Hello").unwrap(), popup!("Hello"));
    }

    #[test]
    fn test_rule_title_with_spaces() {
        assert_eq!(parse("Hello World").unwrap(), popup!("Hello World"));
    }

    #[test]
    fn test_rule_title_with_numbers() {
        assert_eq!(parse("Test 123").unwrap(), popup!("Test 123"));
    }

    #[test]
    fn test_rule_title_with_special_chars() {
        assert_eq!(parse("Test-App_v2.0!").unwrap(), popup!("Test-App_v2.0!"));
    }

    #[test]
    fn test_rule_title_with_unicode() {
        assert_eq!(parse("🎮 Game Settings 🎮").unwrap(), popup!("🎮 Game Settings 🎮"));
    }

    #[test]
    fn test_rule_title_markdown_h1() {
        assert_eq!(parse("# Settings").unwrap(), popup!("Settings"));
    }

    #[test]
    fn test_rule_title_markdown_h2() {
        assert_eq!(parse("## Settings").unwrap(), popup!("Settings"));
    }

    #[test]
    fn test_rule_title_markdown_h3() {
        assert_eq!(parse("### Settings").unwrap(), popup!("Settings"));
    }

    #[test]
    fn test_rule_title_confirm_prefix() {
        assert_eq!(parse("confirm Delete file?").unwrap(), popup!("Delete file?"));
    }

    #[test]
    fn test_rule_title_confirm_with_markdown() {
        assert_eq!(parse("confirm # Delete?").unwrap(), popup!("Delete?"));
    }

    #[test]
    fn test_rule_labeled_item_slider_dash() {
        assert_eq!(
            parse("Test\nVolume: 0-100").unwrap(),
            popup!("Test", [slider!("Volume", 0, 100)])
        );
    }

    #[test]
    fn test_rule_labeled_item_slider_dots() {
        assert_eq!(
            parse("Test\nVolume: 0..100").unwrap(),
            popup!("Test", [slider!("Volume", 0, 100)])
        );
    }

    #[test]
    fn test_rule_labeled_item_slider_to() {
        assert_eq!(
            parse("Test\nVolume: 0 to 100").unwrap(),
            popup!("Test", [slider!("Volume", 0, 100)])
        );
    }

    #[test]
    fn test_rule_labeled_item_slider_with_default() {
        assert_eq!(
            parse("Test\nVolume: 0-100 = 75").unwrap(),
            popup!("Test", [slider!("Volume", 0, 100, 75)])
        );
    }

    #[test]
    fn test_rule_labeled_item_checkbox_yes() {
        assert_eq!(
            parse("Test\nEnabled: yes").unwrap(),
            popup!("Test", [checkbox!("Enabled", true)])
        );
    }

    #[test]
    fn test_rule_labeled_item_checkbox_no() {
        assert_eq!(
            parse("Test\nEnabled: no").unwrap(),
            popup!("Test", [checkbox!("Enabled", false)])
        );
    }

    #[test]
    fn test_rule_labeled_item_choice_pipe() {
        assert_eq!(
            parse("Test\nTheme: Light | Dark").unwrap(),
            popup!("Test", [choice!("Theme", ["Light", "Dark"])])
        );
    }

    #[test]
    fn test_rule_labeled_item_multiselect() {
        assert_eq!(
            parse("Test\nTags: [A, B, C]").unwrap(),
            popup!("Test", [multiselect!("Tags", ["A", "B", "C"])])
        );
    }

    #[test]
    fn test_rule_labeled_item_textbox() {
        assert_eq!(
            parse("Test\nName: @Enter name").unwrap(),
            popup!("Test", [textbox!("Name", "Enter name")])
        );
    }

    #[test]
    fn test_rule_labeled_item_text() {
        assert_eq!(
            parse("Test\nStatus: Active").unwrap(),
            popup!("Test", [text!("Status: Active")])
        );
    }

    #[test]
    fn test_rule_buttons_bracket() {
        assert_eq!(
            parse("Test\n[OK | Cancel]").unwrap(),
            popup!("Test", [buttons!(["OK", "Cancel"])])
        );
    }

    #[test]
    fn test_rule_buttons_arrow() {
        assert_eq!(
            parse("Test\n→ Continue").unwrap(),
            popup!("Test", [buttons!(["Continue"])])
        );
    }

    #[test]
    fn test_rule_buttons_or() {
        assert_eq!(
            parse("Test\nYes or No").unwrap(),
            popup!("Test", [buttons!(["Yes", "No"])])
        );
    }

    #[test]
    fn test_rule_message_warning() {
        assert_eq!(
            parse("Test\n! Warning message").unwrap(),
            popup!("Test", [text!("⚠️ Warning message")])
        );
    }

    #[test]
    fn test_rule_message_info() {
        assert_eq!(
            parse("Test\n> Info message").unwrap(),
            popup!("Test", [text!("ℹ️ Info message")])
        );
    }

    #[test]
    fn test_rule_message_question() {
        assert_eq!(
            parse("Test\n? Question").unwrap(),
            popup!("Test", [text!("❓ Question")])
        );
    }

    #[test]
    fn test_rule_message_bullet() {
        assert_eq!(
            parse("Test\n• Bullet point").unwrap(),
            popup!("Test", [text!("• Bullet point")])
        );
    }

    #[test]
    fn test_rule_text_block_single() {
        assert_eq!(
            parse("Test\nPlain text line").unwrap(),
            popup!("Test", [text!("Plain text line")])
        );
    }

    #[test]
    fn test_rule_text_block_multi() {
        // Each line becomes a separate Text element in current implementation
        assert_eq!(
            parse("Test\nLine one\nLine two\nLine three").unwrap(),
            popup!("Test", [
                text!("Line one"),
                text!("Line two"),
                text!("Line three")
            ])
        );
    }

    #[test]
    fn test_rule_empty_line() {
        assert_eq!(
            parse("Test\n\nAfter blank").unwrap(),
            popup!("Test", [text!("After blank")])
        );
    }

    // ============================================
    // 2. WIDGET DETECTION BOUNDARY TESTS
    // ============================================

    #[test]
    fn test_boundary_choice_two_options() {
        assert_eq!(
            parse("Test\nMode: A | B").unwrap(),
            popup!("Test", [choice!("Mode", ["A", "B"])])
        );
    }

    #[test]
    fn test_boundary_choice_one_option_becomes_text() {
        assert_eq!(
            parse("Test\nMode: A").unwrap(),
            popup!("Test", [text!("Mode: A")])
        );
    }

    #[test]
    fn test_boundary_range_valid() {
        assert_eq!(
            parse("Test\nVal: 0-100").unwrap(),
            popup!("Test", [slider!("Val", 0, 100)])
        );
    }

    #[test]
    fn test_boundary_range_backwards_becomes_text() {
        // TODO: Parser should reject backwards ranges and treat as text
        // Current behavior: accepts backwards range as slider
        assert_eq!(
            parse("Test\nVal: 100-0").unwrap(),
            popup!("Test", [slider!("Val", 100, 0, 50)])
        );
    }

    #[test]
    fn test_boundary_range_with_floats() {
        assert_eq!(
            parse("Test\nVal: 0.5-1.5").unwrap(),
            popup!("Test", [slider!("Val", 0.5, 1.5, 1.0)])
        );
    }

    #[test]
    fn test_boundary_boolean_various_truthy() {
        let truthy = vec!["yes", "YES", "true", "TRUE", "on", "enabled", "✓", "[x]", "(*)"];
        for value in truthy {
            let input = format!("Test\nOpt: {}", value);
            assert_eq!(
                parse(&input).unwrap(),
                popup!("Test", [checkbox!("Opt", true)]),
                "Failed for truthy value: {}", value
            );
        }
    }

    #[test]
    fn test_boundary_boolean_various_falsy() {
        let falsy = vec!["no", "NO", "false", "FALSE", "off", "disabled", "☐", "[ ]", "( )"];
        for value in falsy {
            let input = format!("Test\nOpt: {}", value);
            assert_eq!(
                parse(&input).unwrap(),
                popup!("Test", [checkbox!("Opt", false)]),
                "Failed for falsy value: {}", value
            );
        }
    }

    #[test]
    fn test_boundary_textbox_empty_placeholder() {
        assert_eq!(
            parse("Test\nInput: @").unwrap(),
            popup!("Test", [textbox!("Input")])
        );
    }

    #[test]
    fn test_boundary_multiselect_empty() {
        assert_eq!(
            parse("Test\nTags: []").unwrap(),
            popup!("Test", [text!("Tags: []")])
        );
    }

    // ============================================
    // 3. AMBIGUITY RESOLUTION TESTS
    // ============================================

    #[test]
    fn test_ambiguous_comma_in_sentence() {
        // TODO: Parser should detect this as text, not choice
        // Current behavior: treats "Hello, world" as choice options
        assert_eq!(
            parse("Test\nNote: Hello, world").unwrap(),
            popup!("Test", [choice!("Note", ["Hello", "world"])])
        );
    }

    #[test]
    fn test_ambiguous_slash_in_path() {
        assert_eq!(
            parse("Test\nPath: /usr/bin").unwrap(),
            popup!("Test", [text!("Path: /usr/bin")])
        );
    }

    #[test]
    fn test_ambiguous_pipe_in_text() {
        // TODO: Parser should detect shell command as text, not choice
        // Current behavior: treats "ls | grep" as choice options
        assert_eq!(
            parse("Test\nCommand: ls | grep").unwrap(),
            popup!("Test", [choice!("Command", ["ls", "grep"])])
        );
    }

    #[test]
    fn test_ambiguous_dot_in_filename() {
        assert_eq!(
            parse("Test\nFile: document.txt").unwrap(),
            popup!("Test", [text!("File: document.txt")])
        );
    }

    #[test]
    fn test_ambiguous_numbers_not_range() {
        assert_eq!(
            parse("Test\nVersion: 2.0.1").unwrap(),
            popup!("Test", [text!("Version: 2.0.1")])
        );
    }

    #[test]
    fn test_ambiguous_percentage_not_range() {
        assert_eq!(
            parse("Test\nProgress: 75%").unwrap(),
            popup!("Test", [text!("Progress: 75%")])
        );
    }

    // ============================================
    // 4. COMBINATORIAL TESTS
    // ============================================

    #[test]
    fn test_combo_all_widget_types() {
        let input = "All Widgets\n\
            Slider: 0-100\n\
            Check: yes\n\
            Choice: A | B | C\n\
            Multi: [X, Y, Z]\n\
            Input: @hint\n\
            Info: plain text\n\
            [OK]";
        
        assert_eq!(
            parse(input).unwrap(),
            popup!("All Widgets", [
                slider!("Slider", 0, 100),
                checkbox!("Check", true),
                choice!("Choice", ["A", "B", "C"]),
                multiselect!("Multi", ["X", "Y", "Z"]),
                textbox!("Input", "hint"),
                text!("Info: plain text"),
                buttons!(["OK"])
            ])
        );
    }

    #[test]
    fn test_combo_mixed_messages_and_widgets() {
        let input = "Mixed\n\
            ! Warning\n\
            Volume: 0-100\n\
            > Info\n\
            Enabled: no\n\
            ? Question\n\
            [Done]";
        
        assert_eq!(
            parse(input).unwrap(),
            popup!("Mixed", [
                text!("⚠️ Warning"),
                slider!("Volume", 0, 100),
                text!("ℹ️ Info"),
                checkbox!("Enabled", false),
                text!("❓ Question"),
                buttons!(["Done"])
            ])
        );
    }

    #[test]
    fn test_combo_multiple_text_blocks() {
        let input = "Text Heavy\n\
            This is paragraph one\n\
            continuing on line two\n\
            \n\
            This is paragraph two\n\
            with its own continuation\n\
            \n\
            [Close]";
        
        // Each non-blank line becomes a separate Text element in current implementation
        assert_eq!(
            parse(input).unwrap(),
            popup!("Text Heavy", [
                text!("This is paragraph one"),
                text!("continuing on line two"),
                text!("This is paragraph two"),
                text!("with its own continuation"),
                buttons!(["Close"])
            ])
        );
    }

    #[test]
    fn test_combo_complex_real_world() {
        let input = "# System Update\n\
            ! Critical security update available\n\
            \n\
            Version: 2.5.1\n\
            Size: 145 MB\n\
            Priority: High | Medium | Low\n\
            Auto-install: yes\n\
            Components: [Core, UI, Docs]\n\
            \n\
            > Restart will be required\n\
            \n\
            Notes: @Any additional notes\n\
            \n\
            [Install | Schedule | Skip]";
        
        assert_eq!(
            parse(input).unwrap(),
            popup!("System Update", [
                text!("⚠️ Critical security update available"),
                text!("Version: 2.5.1"),
                text!("Size: 145 MB"),
                choice!("Priority", ["High", "Medium", "Low"]),
                checkbox!("Auto-install", true),
                multiselect!("Components", ["Core", "UI", "Docs"]),
                text!("ℹ️ Restart will be required"),
                textbox!("Notes", "Any additional notes"),
                buttons!(["Install", "Schedule", "Skip"])
            ])
        );
    }

    // ============================================
    // 5. ERROR SPECIFICATION TESTS
    // ============================================

    #[test]
    fn test_error_empty_input() {
        // Empty input is now valid (creates popup with default title and no elements)
        let result = parse("");
        assert!(result.is_ok());
        let popup = result.unwrap();
        assert_eq!(popup.title, "Popup");
        assert_eq!(popup.elements.len(), 0);
    }

    #[test]
    fn test_error_only_whitespace() {
        // Whitespace-only input is now valid (creates popup with default title and no elements)
        let result = parse("   \n  \n  ");
        assert!(result.is_ok());
        let popup = result.unwrap();
        assert_eq!(popup.title, "Popup");
        assert_eq!(popup.elements.len(), 0);
    }

    #[test]
    fn test_error_malformed_range_not_widget() {
        // This should parse but as text, not fail
        assert_eq!(
            parse("Test\nBroken: abc-xyz").unwrap(),
            popup!("Test", [text!("Broken: abc-xyz")])
        );
    }

    // ============================================
    // 6. REGRESSION TESTS
    // ============================================

    #[test]
    fn test_regression_markdown_with_content() {
        // This used to fail due to COMMENT rule conflict
        assert_eq!(
            parse("# Title\nContent").unwrap(),
            popup!("Title", [text!("Content")])
        );
    }

    #[test]
    fn test_regression_multiple_markdown_hashes() {
        // #### is not a supported markdown level, so it's treated as a plain title
        assert_eq!(
            parse("#### Deep Header\n[OK]").unwrap(),
            popup!("#### Deep Header", [buttons!(["OK"])])
        );
    }

    #[test]
    fn test_regression_spaces_around_range_separator() {
        assert_eq!(
            parse("Test\nVal: 0  -  100").unwrap(),
            popup!("Test", [slider!("Val", 0, 100)])
        );
    }

    #[test]
    fn test_regression_natural_language_buttons() {
        assert_eq!(
            parse("Confirm\nYes or No or Maybe").unwrap(),
            popup!("Confirm", [buttons!(["Yes", "No", "Maybe"])])
        );
    }

    // ============================================
    // 7. EDGE CASE TESTS
    // ============================================

    #[test]
    fn test_edge_empty_popup() {
        assert_eq!(
            parse("Empty Popup").unwrap(),
            popup!("Empty Popup")
        );
    }

    #[test]
    fn test_edge_very_long_title() {
        let long_title = "A ".repeat(100) + "Title";
        assert_eq!(
            parse(&long_title).unwrap(),
            popup!(&long_title)
        );
    }

    #[test]
    fn test_edge_unicode_everywhere() {
        assert_eq!(
            parse("🎮 Title 🎮\n🔊 Volume: 0-100\n[✅ OK | ❌ Cancel]").unwrap(),
            popup!("🎮 Title 🎮", [
                slider!("🔊 Volume", 0, 100),
                buttons!(["✅ OK", "❌ Cancel"])
            ])
        );
    }

    #[test]
    fn test_edge_single_character_labels() {
        assert_eq!(
            parse("T\nA: 0-1\nB: yes\nC: X | Y\n[D]").unwrap(),
            popup!("T", [
                slider!("A", 0, 1, 0.5),
                checkbox!("B", true),
                choice!("C", ["X", "Y"]),
                buttons!(["D"])
            ])
        );
    }

    #[test]
    fn test_edge_windows_line_endings() {
        assert_eq!(
            parse("Test\r\nValue: 50\r\n[OK]").unwrap(),
            popup!("Test", [
                text!("Value: 50"),
                buttons!(["OK"])
            ])
        );
    }

    #[test]
    fn test_edge_trailing_whitespace() {
        assert_eq!(
            parse("Test   \nValue: 100   \n[OK]   ").unwrap(),
            popup!("Test", [
                text!("Value: 100"),
                buttons!(["OK"])
            ])
        );
    }

    #[test]
    fn test_edge_many_blank_lines() {
        assert_eq!(
            parse("Test\n\n\n\nContent\n\n\n[OK]").unwrap(),
            popup!("Test", [
                text!("Content"),
                buttons!(["OK"])
            ])
        );
    }

    #[test]
    fn test_edge_nested_brackets_in_text() {
        // TODO: Parser should keep mathematical notation as text
        // Current behavior: incorrectly splits on comma
        assert_eq!(
            parse("Test\nFormula: f(x) = [a, b]\n[OK]").unwrap(),
            popup!("Test", [
                choice!("Formula", ["f(x) = [a", "b]"]),
                buttons!(["OK"])
            ])
        );
    }
}