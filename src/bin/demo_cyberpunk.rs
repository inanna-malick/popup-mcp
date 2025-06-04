use popup_mcp::models::{Element, PopupDefinition};
use popup_mcp::gui::render_popup;

fn main() {
    let definition = PopupDefinition {
        title: "NEURAL INTERFACE v2.0".to_string(),
        elements: vec![
            Element::Text("SYSTEM DIAGNOSTICS".to_string()),
            
            Element::Slider {
                label: "Neural Sync Rate".to_string(),
                min: 0.0,
                max: 100.0,
                default: 75.0,
            },
            Element::Slider {
                label: "Memory Buffer".to_string(),
                min: 0.0,
                max: 10.0,
                default: 7.0,
            },
            
            Element::Checkbox {
                label: "Enhanced Cognition Mode".to_string(),
                default: true,
            },
            Element::Checkbox {
                label: "Security Protocols Active".to_string(),
                default: false,
            },
            
            Element::Choice {
                label: "Operation Mode".to_string(),
                options: vec![
                    "Stealth Operations".to_string(),
                    "Combat Ready".to_string(),
                    "Data Mining".to_string(),
                    "System Maintenance".to_string(),
                ],
            },
            
            Element::Multiselect {
                label: "Active Subsystems".to_string(),
                options: vec![
                    "Neural Core".to_string(),
                    "Quantum Processor".to_string(),
                    "Memory Banks".to_string(),
                    "Security Matrix".to_string(),
                ],
            },
            
            Element::Group {
                label: "Network Status".to_string(),
                elements: vec![
                    Element::Checkbox {
                        label: "Connected to Grid".to_string(),
                        default: true,
                    },
                    Element::Slider {
                        label: "Bandwidth".to_string(),
                        min: 0.0,
                        max: 1000.0,
                        default: 750.0,
                    },
                ],
            },
            
            Element::Textbox {
                label: "System Notes".to_string(),
                placeholder: Some("Enter observations...".to_string()),
                rows: Some(2),
            },
            
            Element::Buttons(vec![
                "Execute".to_string(),
                "Continue".to_string(),
                "Abort Mission".to_string(),
                "Force Yield".to_string(),
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