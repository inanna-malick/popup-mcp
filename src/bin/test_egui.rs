use popup_mcp::models::{Element, PopupDefinition};
use popup_mcp::gui::render_popup;

fn main() {
    let definition = PopupDefinition {
        title: "Test Popup".to_string(),
        elements: vec![
            Element::Text("Testing egui implementation".to_string()),
            Element::Slider {
                label: "Energy".to_string(),
                min: 0.0,
                max: 10.0,
                default: 5.0,
            },
            Element::Checkbox {
                label: "Ready to proceed".to_string(),
                default: false,
            },
            Element::Textbox {
                label: "Comments".to_string(),
                placeholder: Some("Enter any comments...".to_string()),
                rows: Some(2),
            },
            Element::Choice {
                label: "Select option".to_string(),
                options: vec![
                    "Option A".to_string(),
                    "Option B".to_string(),
                    "Option C".to_string(),
                ],
            },
            Element::Buttons(vec![
                "Submit".to_string(),
                "Cancel".to_string(),
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