use anyhow::Result;
use clap::Parser;
use popup_mcp::{parse_popup_json, render_popup, mcp_server};
use std::fs;
use std::io::{self, Read};

#[derive(Parser)]
#[command(name = "popup")]
#[command(about = "Native GUI popups with MCP server support", long_about = None)]
struct Args {
    /// Test mode: read JSON and show popup
    #[arg(long)]
    test: bool,
    
    /// Input file for test mode (reads stdin if not provided)
    input_file: Option<String>,
    
    /// Include only these templates (comma-separated)
    #[arg(long, value_delimiter = ',')]
    include_only: Option<Vec<String>>,
    
    /// Exclude these templates (comma-separated)  
    #[arg(long, value_delimiter = ',')]
    exclude: Option<Vec<String>>,
    
    /// List available templates and exit
    #[arg(long)]
    list_templates: bool,
}

fn run_test_mode(input_file: Option<String>) -> Result<()> {
    // Read input from file or stdin
    let input = if let Some(path) = input_file {
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

fn main() -> Result<()> {
    let args = Args::parse();
    
    if args.test {
        // Test mode: read JSON and show popup
        run_test_mode(args.input_file)
    } else {
        // MCP server mode
        let server_args = mcp_server::ServerArgs {
            include_only: args.include_only,
            exclude: args.exclude,
            list_templates: args.list_templates,
        };
        mcp_server::run(server_args)
    }
}