use anyhow::Result;
use clap::{Parser, Subcommand};
use popup_mcp::{parse_popup_dsl, render_popup};
use std::fs;

#[derive(Parser)]
#[command(name = "spike")]
#[command(about = "Structured popup interface for high-bandwidth human→AI communication")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show a popup from a .popup file
    Show {
        /// Path to the .popup file
        file: String,
    },
    /// Quick check-in for energy and clarity
    Checkin,
    /// Get feedback on a decision or approach
    Feedback {
        /// Context for the feedback request
        #[arg(short, long)]
        context: Option<String>,
    },
    /// Quick triage for priority decisions
    Triage {
        /// Items to triage (comma-separated)
        items: Vec<String>,
    },
    /// Run a popup defined inline
    Run {
        /// Popup DSL code
        dsl: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::Show { file }) => {
            let content = fs::read_to_string(&file)?;
            let definition = parse_popup_dsl(&content)?;
            render_popup(definition)?
        }
        
        Some(Commands::Checkin) => {
            let dsl = r#"
popup "System Check-in" [
    text "How are you doing right now?"
    
    slider "Energy" 0..10 default = 5
    slider "Clarity" 0..10 default = 5
    checkbox "Fog present" default = false
    checkbox "Body needs first" default = false
    
    textbox "Other observations"
    
    buttons ["Continue", "Take Break", "Force Yield"]
]
"#;
            let definition = parse_popup_dsl(dsl)?;
            render_popup(definition)?
        }
        
        Some(Commands::Feedback { context }) => {
            let context_text = context.unwrap_or_else(|| "Current approach".to_string());
            let dsl = format!(r#"
popup "Feedback Request" [
    text "Context: {}"
    
    choice "How does this approach feel?" [
        "Good - proceed",
        "Uncertain - discuss more",
        "Wrong direction - rethink"
    ]
    
    slider "Confidence" 0..10 default = 5
    
    textbox "Specific concerns or suggestions" rows=3
    
    textbox "Other observations"
    
    buttons ["Submit", "Force Yield"]
]
"#, context_text);
            let definition = parse_popup_dsl(&dsl)?;
            render_popup(definition)?
        }
        
        Some(Commands::Triage { items }) => {
            if items.is_empty() {
                eprintln!("Error: No items provided for triage");
                std::process::exit(1);
            }
            
            let items_text = items.join("\n    - ");
            let dsl = format!(r#"
popup "Quick Triage" [
    text "Items to triage:"
    text "    - {}"
    
    choice "Priority action:" [
        "Do first item now",
        "Do second item now",
        "Do third item now",
        "Defer all",
        "Need more context"
    ]
    
    checkbox "Time sensitive" default = false
    
    textbox "Other observations"
    
    buttons ["Execute", "Delegate", "Delete", "Force Yield"]
]
"#, items_text);
            let definition = parse_popup_dsl(&dsl)?;
            render_popup(definition)?
        }
        
        Some(Commands::Run { dsl }) => {
            let definition = parse_popup_dsl(&dsl)?;
            render_popup(definition)?
        }
        
        None => {
            // Default behavior - show a general purpose feedback popup
            let dsl = r#"
popup "Spike Feedback" [
    text "What would you like to communicate?"
    
    choice "Type:" [
        "Observation",
        "Correction",
        "Preference",
        "Question",
        "Other"
    ]
    
    textbox "Message" rows=3
    
    slider "Importance" 0..10 default = 5
    
    textbox "Other observations"
    
    buttons ["Send", "Cancel", "Force Yield"]
]
"#;
            let definition = parse_popup_dsl(dsl)?;
            render_popup(definition)?
        }
    };
    
    // Output result as JSON for easy parsing
    println!("{}", serde_json::to_string_pretty(&result)?);
    
    Ok(())
}