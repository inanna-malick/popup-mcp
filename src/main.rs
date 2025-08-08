use anyhow::Result;
use clap::Parser;
use popup_mcp::{parse_popup_json, render_popup};
use std::fs;
use std::io::{self, Read};

#[derive(Parser)]
#[command(name = "popup-mcp")]
#[command(about = "Create native GUI popups from JSON", long_about = None)]
struct Cli {
    /// Optional input file (reads from stdin if not provided)
    input_file: Option<String>,
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
    
    // Parse JSON and render popup
    match parse_popup_json(&input).and_then(render_popup) {
        Ok(result) => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Err(e) => {
            let error = serde_json::json!({"error": e.to_string()});
            println!("{}", serde_json::to_string(&error)?);
        }
    }
    
    Ok(())
}