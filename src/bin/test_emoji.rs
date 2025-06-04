use popup_mcp::models::{Element, PopupDefinition};
use popup_mcp::gui::render_popup;

fn main() {
    let definition = PopupDefinition {
        title: "Emoji Test 🎉".to_string(),
        elements: vec![
            Element::Text("Testing emoji support in egui! 🚀".to_string()),
            Element::Text("Various emojis: 😀 😎 🔥 ✨ 🌟".to_string()),
            Element::Slider {
                label: "Energy ⚡".to_string(),
                min: 0.0,
                max: 10.0,
                default: 5.0,
            },
            Element::Checkbox {
                label: "Ready to go? 🏃‍♂️".to_string(),
                default: false,
            },
            Element::Choice {
                label: "Select mood:".to_string(),
                options: vec![
                    "Happy 😊".to_string(),
                    "Excited 🤩".to_string(),
                    "Focused 🎯".to_string(),
                    "Relaxed 😌".to_string(),
                ],
            },
            Element::Textbox {
                label: "Comments 💭".to_string(),
                placeholder: Some("Enter your thoughts... 💡".to_string()),
                rows: Some(2),
            },
            Element::Buttons(vec![
                "Submit ✅".to_string(),
                "Cancel ❌".to_string(),
                "Force Yield 🛑".to_string(),
            ]),
        ],
    };
    
    match render_popup(definition) {
        Ok(result) => {
            println!("Result: {:?}", result);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}