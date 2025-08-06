use pest::Parser;
use popup_mcp::dsl::simple_parser::{SimpleParser, Rule};


#[test]
fn test_condition_parses() {
    let result = SimpleParser::parse(Rule::condition, "Advanced");
    assert!(result.is_ok(), "condition should parse 'Advanced'");
    
    let pairs: Vec<_> = result.unwrap().collect();
    assert_eq!(pairs.len(), 1);
    assert_eq!(pairs[0].as_rule(), Rule::condition);
    assert_eq!(pairs[0].as_str(), "Advanced");
}


#[test]
fn test_full_conditional_works() {
    // Test empty conditional
    let result = SimpleParser::parse(Rule::conditional, "[if Advanced] {}");
    assert!(result.is_ok(), "conditional should parse '[if Advanced] {{}}'");
    
    let pairs: Vec<_> = result.unwrap().collect();
    assert_eq!(pairs.len(), 1);
    assert_eq!(pairs[0].as_rule(), Rule::conditional);
    assert_eq!(pairs[0].as_str(), "[if Advanced] {}");
    
    // Test conditional with content
    let result_with_content = SimpleParser::parse(Rule::conditional, "[if Advanced] {Debug: yes}");
    assert!(result_with_content.is_ok(), "conditional with content should parse");
    
    let pairs_with_content: Vec<_> = result_with_content.unwrap().collect();
    assert_eq!(pairs_with_content.len(), 1);
    assert_eq!(pairs_with_content[0].as_rule(), Rule::conditional);
    assert_eq!(pairs_with_content[0].as_str(), "[if Advanced] {Debug: yes}");
}

#[test]
fn test_conditional_as_element_works() {
    // Test that conditional parses correctly as an element
    let result = SimpleParser::parse(Rule::element, "[if Advanced] {}");
    assert!(result.is_ok(), "conditional should parse as element");
    
    let pairs: Vec<_> = result.unwrap().collect();
    assert_eq!(pairs.len(), 1);
    assert_eq!(pairs[0].as_rule(), Rule::element);
    
    // Should have conditional inside
    let inner: Vec<_> = pairs[0].clone().into_inner().collect();
    assert_eq!(inner.len(), 1);
    assert_eq!(inner[0].as_rule(), Rule::conditional);
    assert_eq!(inner[0].as_str(), "[if Advanced] {}");
}