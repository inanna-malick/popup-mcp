#[cfg(test)]
mod tests {
    use pest::Parser;
    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "src/simple.pest"]
    pub struct ConditionalTestParser;


    #[test]
    fn test_simple_condition() {
        // Test basic condition parsing
        let test_cases = vec![
            "Advanced",
            "not Advanced", 
            "Theme = Dark",
            "Count > 5",
        ];

        for case in test_cases {
            let result = ConditionalTestParser::parse(Rule::condition, case);
            assert!(result.is_ok(), "Condition '{}' should parse", case);
        }
    }

    #[test]
    fn test_conditional_with_content() {
        // Test empty conditional 
        let result = ConditionalTestParser::parse(Rule::conditional, "[if Advanced] {}");
        assert!(result.is_ok(), "Empty conditional should parse");

        // Test conditional with elements
        println!("Testing conditional with content: '[if Advanced] {{Debug: yes}}'");
        let result = ConditionalTestParser::parse(Rule::conditional, "[if Advanced] {Debug: yes}");
        match &result {
            Ok(pairs) => {
                println!("✅ Conditional body parsed");
                for pair in pairs.clone() {
                    print_parse_tree(pair, 0);
                }
            }
            Err(e) => {
                println!("❌ Conditional body error: {}", e);
            }
        }
        assert!(result.is_ok(), "Conditional body with element should parse");
    }

    #[test]
    fn test_conditional_step_by_step() {
        // Test each part of the conditional separately
        
        // 1. Test full conditional
        println!("1. Testing complete conditional '[if Advanced] {{}}'");
        let complete_result = ConditionalTestParser::parse(Rule::conditional, "[if Advanced] {}");
        assert!(complete_result.is_ok(), "complete conditional should work");

        // 2. Test condition 
        println!("2. Testing condition 'Advanced'");
        let condition_result = ConditionalTestParser::parse(Rule::condition, "Advanced");
        assert!(condition_result.is_ok(), "condition should work");

        // 3. Test full conditional with empty body
        println!("3. Testing empty conditional '[if Advanced] {{}}'");
        let empty_result = ConditionalTestParser::parse(Rule::conditional, "[if Advanced] {}");
        assert!(empty_result.is_ok(), "empty conditional should work");
        
        // 4. Test full conditional with content
        println!("4. Testing conditional with content '[if Advanced] {{Debug: yes}}'");
        let content_result = ConditionalTestParser::parse(Rule::conditional, "[if Advanced] {Debug: yes}");
        assert!(content_result.is_ok(), "conditional with content should work");
        
        // 5. Test the minimal complete conditional
        println!("5. Testing minimal '[if Advanced] {{}}'");
        let minimal_result = ConditionalTestParser::parse(Rule::conditional, "[if Advanced] {}");
        match &minimal_result {
            Ok(pairs) => {
                println!("✅ Minimal conditional parsed");
                for pair in pairs.clone() {
                    print_parse_tree(pair, 0);
                }
            }
            Err(e) => {
                println!("❌ Minimal conditional error: {}", e);
            }
        }
        
        // 6. Let's also test if the issue is with EOI (end of input)
        println!("6. Testing with explicit EOI");
        let eoi_result = ConditionalTestParser::parse(Rule::conditional, "[if Advanced] {}");
        println!("EOI result: {:?}", eoi_result.is_ok());
        
        // 7. Test if the issue is with parsing
        println!("7. Testing conditional with content");
        let with_content_result = ConditionalTestParser::parse(Rule::conditional, "[if Advanced] {Debug: yes}");
        println!("With content result: {:?}", with_content_result.is_ok());
        
        assert!(minimal_result.is_ok(), "Minimal conditional should parse");
    }

    #[test]
    fn test_full_conditional() {
        // Test complete conditional syntax
        let test_cases = vec![
            "[if Advanced] { Debug: yes }",
            "[if Theme = Dark] { Contrast: high }",
            "[if not Enabled] { Warning: disabled }",
        ];

        for case in test_cases {
            println!("Testing conditional: {}", case);
            let result = ConditionalTestParser::parse(Rule::conditional, case);
            match &result {
                Ok(pairs) => {
                    println!("✅ Parsed successfully");
                    for pair in pairs.clone() {
                        println!("  Rule: {:?}, Text: '{}'", pair.as_rule(), pair.as_str());
                    }
                }
                Err(e) => {
                    println!("❌ Parse error: {}", e);
                }
            }
            assert!(result.is_ok(), "Conditional '{}' should parse", case);
        }
    }

    #[test]
    fn test_conditional_vs_bracket_buttons() {
        // Test that conditionals are distinguished from bracket buttons
        
        // This should parse as conditional
        let conditional_result = ConditionalTestParser::parse(Rule::conditional, "[if X] { Y: yes }");
        assert!(conditional_result.is_ok(), "[if X] should parse as conditional");

        // This should parse as bracket buttons
        let button_result = ConditionalTestParser::parse(Rule::bracket_buttons, "[OK | Cancel]");
        assert!(button_result.is_ok(), "[OK | Cancel] should parse as bracket buttons");

        // This should NOT parse as bracket buttons (starts with 'if')
        let not_button_result = ConditionalTestParser::parse(Rule::bracket_buttons, "[if X]");
        assert!(not_button_result.is_err(), "[if X] should NOT parse as bracket buttons");
    }

    #[test]
    fn test_element_parsing_priority() {
        // Test which rule gets chosen for ambiguous input
        
        let test_cases = vec![
            ("[if X] { Y: yes }", "Should parse as conditional"),
            ("[OK | Cancel]", "Should parse as buttons"),
            ("! Warning", "Should parse as message"),
            ("Label: value", "Should parse as labeled_item"),
        ];

        for (input, description) in test_cases {
            println!("Testing element: {} ({})", input, description);
            let result = ConditionalTestParser::parse(Rule::element, input);
            match &result {
                Ok(pairs) => {
                    let pair = pairs.clone().next().unwrap();
                    println!("✅ Parsed as: {:?}", pair.as_rule());
                    // Check which inner rule was matched
                    if let Some(inner) = pair.into_inner().next() {
                        println!("  Inner rule: {:?}", inner.as_rule());
                    }
                }
                Err(e) => {
                    println!("❌ Parse error: {}", e);
                }
            }
            assert!(result.is_ok(), "Element '{}' should parse", input);
        }
    }

    #[test]
    fn test_popup_with_conditional() {
        // Test complete popup with conditional
        let popup_input = "Test\n[if Advanced] { Debug: yes }\n[OK]";
        
        println!("Testing full popup:\n{}", popup_input);
        let result = ConditionalTestParser::parse(Rule::popup, popup_input);
        
        match &result {
            Ok(pairs) => {
                println!("✅ Popup parsed successfully");
                for pair in pairs.clone() {
                    print_parse_tree(pair, 0);
                }
            }
            Err(e) => {
                println!("❌ Popup parse error: {}", e);
            }
        }
        
        assert!(result.is_ok(), "Full popup with conditional should parse");
    }

    #[test]
    fn test_nested_conditionals() {
        // Test nested conditional structure
        let nested_input = "[if A] { B: yes [if B] { C: no } }";
        
        println!("Testing nested conditional: {}", nested_input);
        let result = ConditionalTestParser::parse(Rule::conditional, nested_input);
        
        match &result {
            Ok(pairs) => {
                println!("✅ Nested conditional parsed");
                for pair in pairs.clone() {
                    print_parse_tree(pair, 0);
                }
            }
            Err(e) => {
                println!("❌ Nested conditional error: {}", e);
            }
        }
        
        assert!(result.is_ok(), "Nested conditional should parse");
    }

    // Helper function to print parse tree
    fn print_parse_tree(pair: pest::iterators::Pair<Rule>, indent: usize) {
        let indent_str = "  ".repeat(indent);
        println!("{}Rule: {:?}, Text: '{}'", indent_str, pair.as_rule(), pair.as_str());
        
        for inner in pair.into_inner() {
            print_parse_tree(inner, indent + 1);
        }
    }
}