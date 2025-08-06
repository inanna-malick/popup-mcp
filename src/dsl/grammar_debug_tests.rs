#[cfg(test)]
mod tests {
    use pest::Parser;
    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "src/simple.pest"]
    pub struct TestParser;

    #[test]
    fn test_popup_accepts_empty() {
        // Empty popup should be valid
        let result = TestParser::parse(Rule::popup, "");
        assert!(result.is_ok(), "empty popup should parse");
    }

    #[test]
    fn test_popup_accepts_single_element() {
        let cases = vec![
            "Debug: yes",
            "[OK | Cancel]",
            "! Warning message",
            "Some text",
        ];
        
        for case in cases {
            let result = TestParser::parse(Rule::popup, case);
            assert!(result.is_ok(), "popup failed for single element: {:?}", case);
        }
    }

    #[test]
    fn test_popup_accepts_multiple_elements() {
        let input = "Debug: yes\nVolume: 0-100\n[Save | Cancel]";
        let result = TestParser::parse(Rule::popup, input);
        assert!(result.is_ok(), "popup should accept multiple elements");
    }

    #[test]
    fn test_element_types() {
        // Test that various element types parse correctly
        let test_cases = vec![
            ("Label: value", "labeled_item"),
            ("[OK | Cancel]", "buttons"),
            ("! Warning", "message"),
            ("[if X] { Y: yes }", "conditional"),
            ("Plain text", "text_block"),
        ];
        
        for (input, expected_type) in test_cases {
            let result = TestParser::parse(Rule::element, input);
            assert!(result.is_ok(), "element failed for {}: {:?}", expected_type, input);
            
            let element = result.unwrap().next().unwrap();
            let inner = element.into_inner().next().unwrap();
            
            // Check that it parsed as the expected type
            let rule_name = format!("{:?}", inner.as_rule());
            // Remove quotes from Debug format if present
            let rule_name_clean = rule_name.trim_matches('"');
            // Compare both with underscores removed for flexibility
            assert!(rule_name_clean.replace("_", "").to_lowercase().contains(&expected_type.replace("_", "").to_lowercase()), 
                "Expected {} but got {} for input: {}", expected_type, rule_name_clean, input);
        }
    }

    #[test]
    fn test_conditional_vs_buttons() {
        // Test that [if X] is parsed as conditional, not buttons
        let cond_result = TestParser::parse(Rule::conditional, "[if Advanced] { Debug: yes }");
        assert!(cond_result.is_ok(), "conditional should parse");
        
        // Test that [OK] is parsed as buttons, not conditional
        let button_result = TestParser::parse(Rule::bracket_buttons, "[OK | Cancel]");
        assert!(button_result.is_ok(), "bracket_buttons should parse");
        
        // Test that [if X] is NOT parsed as buttons due to negative lookahead
        let not_button = TestParser::parse(Rule::bracket_buttons, "[if X]");
        assert!(not_button.is_err(), "[if X] should NOT parse as bracket_buttons");
    }

    #[test]
    fn test_labeled_item_values() {
        // Test various value patterns in labeled items
        let test_cases = vec![
            "Label: yes",           // boolean
            "Label: 0-100",        // range
            "Label: A | B | C",    // choice
            "Label: plain text",   // text
            "Label: @placeholder", // textbox hint
        ];
        
        for case in test_cases {
            let result = TestParser::parse(Rule::labeled_item, case);
            assert!(result.is_ok(), "labeled_item failed for: {:?}", case);
        }
    }

    #[test]
    fn test_message_prefixes() {
        let prefixes = vec![
            ("> Info message", ">"),
            ("! Warning message", "!"),
            ("? Question", "?"),
            ("• Bullet point", "•"),
        ];
        
        for (input, prefix) in prefixes {
            let result = TestParser::parse(Rule::message, input);
            assert!(result.is_ok(), "message failed for prefix {}: {:?}", prefix, input);
            
            let message = result.unwrap().next().unwrap();
            let mut inner = message.into_inner();
            let prefix_node = inner.next().unwrap();
            
            assert_eq!(prefix_node.as_str(), prefix, "Wrong prefix parsed");
        }
    }

    #[test]
    fn test_button_formats() {
        // Test different button formats
        let test_cases = vec![
            ("[OK | Cancel]", Rule::bracket_buttons),
            ("→ Continue", Rule::arrow_button),
            ("Yes or No", Rule::or_buttons),
        ];
        
        for (input, expected_rule) in test_cases {
            let result = TestParser::parse(expected_rule, input);
            assert!(result.is_ok(), "{:?} failed for: {:?}", expected_rule, input);
        }
    }

    #[test]
    fn test_condition_types() {
        let test_cases = vec![
            ("Advanced", "simple_condition"),
            ("not Advanced", "not_condition"),
            ("Tags has Work", "has_condition"),
            ("Count > 5", "compare_condition"),
            ("Theme = Dark", "compare_condition"),
        ];
        
        for (input, condition_type) in test_cases {
            let result = TestParser::parse(Rule::condition, input);
            assert!(result.is_ok(), "condition failed for {}: {:?}", condition_type, input);
        }
    }

    #[test]
    fn test_nested_conditionals() {
        // Test that conditionals can contain other elements including conditionals
        let nested = "[if A] { B: yes\n[if B] { C: no } }";
        let result = TestParser::parse(Rule::conditional, nested);
        assert!(result.is_ok(), "nested conditional should parse");
    }

    #[test]
    fn test_multiline_popup() {
        let input = r#"! Important Notice
This is a multi-line
text block that should parse

Settings:
Volume: 0-100
Theme: Light | Dark

[if Advanced] {
    Debug: yes
    Log Level: Info | Warning | Error
}

[Save | Cancel]"#;
        
        let result = TestParser::parse(Rule::popup, input);
        assert!(result.is_ok(), "complex multiline popup should parse");
    }

    #[test]
    fn test_blank_lines_allowed() {
        // Blank lines should be handled by whitespace
        let input = "Item 1: yes\n\nItem 2: no\n\n[OK]";
        let result = TestParser::parse(Rule::popup, input);
        assert!(result.is_ok(), "popup with blank lines should parse");
    }
}