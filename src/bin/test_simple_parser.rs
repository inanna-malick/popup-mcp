use popup_mcp::parse_popup_dsl;
use std::fs;

fn main() {
    let examples = vec![
        ("Simple confirmation", r#"confirm Delete file?
Yes or No"#),

        ("Settings", r#"Settings
Volume: 0-100 = 75
Theme: Light | Dark
Notifications: yes
Auto-save: enabled
Language: English
[Save | Cancel]"#),

        ("Messages", r#"System Update
! Critical security update
> Download size: 145MB
? Need help?
• Restart required
Plain text here
[Install | Later]"#),
    ];

    for (name, input) in examples {
        println!("\n=== {} ===", name);
        println!("Input:\n{}\n", input);
        
        match parse_popup_dsl(input) {
            Ok(popup) => {
                println!("✓ Parsed successfully!");
                println!("Title: {}", popup.title);
                println!("Elements: {} items", popup.elements.len());
                for (i, element) in popup.elements.iter().enumerate() {
                    println!("  [{}] {:?}", i, element);
                }
            }
            Err(e) => {
                println!("✗ Parse error: {}", e);
            }
        }
    }
}