use anyhow::Result;
use clap::Parser;
use popup_mcp::{parse_popup_dsl_with_title, render_popup};
use std::fs;
use std::io::{self, Read};

#[derive(Parser)]
#[command(name = "popup-mcp")]
#[command(about = "Create native GUI popups from DSL", long_about = None)]
struct Cli {
    /// Optional input file (reads from stdin if not provided)
    input_file: Option<String>,
    
    /// Title for the popup window
    #[arg(short, long)]
    title: Option<String>,
    
    /// Validate mode - parse and output AST without rendering
    #[arg(long)]
    validate: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Read input from file or stdin
    let input = if let Some(path) = cli.input_file {
        fs::read_to_string(path)?
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        buf
    };
    
    if cli.validate {
        // Validate and output raw AST
        match parse_popup_dsl_with_title(&input, cli.title.clone()) {
            Ok(popup_def) => {
                println!("✅ Valid DSL");
                println!("Raw AST: {:#?}", popup_def);
            }
            Err(e) => {
                println!("❌ Invalid DSL");
                println!("Error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Parse and render popup (normal mode)
        match parse_popup_dsl_with_title(&input, cli.title).and_then(render_popup) {
            Ok(result) => {
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
            Err(e) => {
                let error = serde_json::json!({"error": e.to_string()});
                println!("{}", serde_json::to_string(&error)?);
            }
        }
    }
    
    Ok(())
}