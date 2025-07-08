use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "unified.pest"]
pub struct UnifiedParser;

#[test]
fn test_all_examples_parse() {
    let examples = vec![
        // Natural language
        r#"confirm Delete file? with Yes or No"#,
        
        // Structured with colon
        r#"Settings:
  Volume: 0-100 = 75
  Theme: Light | Dark
  Notifications: ✓
  [Save | Cancel]"#,
        
        // Structured without colon
        r#"Quick Setup
  Name: @Your name
  → Continue"#,
        
        // Mixed styles
        r#"confirm Save changes?
  Modified files: 3
  Size: 1.2MB
  with Save or Discard"#,
        
        // Conditionals
        r#"Account Type:
  Plan: Free | Pro
  when Plan = Pro:
    Features: [API, Analytics, Support]
    Billing: Monthly | Yearly
  [Continue]"#,
        
        // Text messages
        r#"System Alert:
  ! Critical update required
  > Download size: 145MB
  ? Restart will be needed
  [Install Now | Later]"#,
        
        // Sections
        r#"Preferences:
  --- Display ---
  Theme: Light | Dark
  Font Size: 10-20 = 14
  
  --- Behavior ---
  Auto-save: yes
  Confirm exit: no
  
  Save or Cancel"#,
        
        // All button formats
        r#"Button Tests:
  Test 1: [A | B | C]
  Test 2: → Next
  Test 3: buttons: Submit or Reset
  Test 4: with Accept or Decline
  ---
  Done | Exit"#,
    ];
    
    for (i, example) in examples.iter().enumerate() {
        println!("\n=== Example {} ===", i + 1);
        println!("{}", example);
        println!("\nParsing result:");
        
        match UnifiedParser::parse(Rule::popup, example) {
            Ok(pairs) => {
                println!("✓ SUCCESS");
                // Count elements
                let mut element_count = 0;
                for pair in pairs {
                    count_elements(&pair, &mut element_count);
                }
                println!("  Found {} elements", element_count);
            }
            Err(e) => {
                println!("✗ FAILED: {}", e);
            }
        }
    }
}

fn count_elements(pair: &pest::iterators::Pair<Rule>, count: &mut usize) {
    if pair.as_rule() == Rule::element {
        *count += 1;
    }
    for inner in pair.clone().into_inner() {
        count_elements(&inner, count);
    }
}