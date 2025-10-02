use anyhow::Result;
use clap::Parser;
use popup_gui::{mcp_server, parse_popup_json, render_popup};
use std::fs;
use std::io::{self, Read};

#[derive(Parser)]
#[command(name = "popup")]
#[command(about = "Native GUI popups with MCP server support", long_about = None)]
struct Args {
    /// Read JSON from stdin and show popup
    #[arg(long)]
    stdin: bool,

    /// Read JSON from file and show popup
    #[arg(long, value_name = "PATH")]
    file: Option<String>,

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

fn run_stdin_mode() -> Result<()> {
    // Read JSON from stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    // Parse JSON and render popup
    match parse_popup_json(&input).and_then(render_popup) {
        Ok(result) => {
            println!("{}", serde_json::to_string_pretty(&result)?);
            std::process::exit(0); // Success - user interaction completed
        }
        Err(e) => {
            let error = serde_json::json!({"error": e.to_string()});
            println!("{}", serde_json::to_string(&error)?);
            std::process::exit(1); // Error - parsing or rendering failed
        }
    }
}

fn run_file_mode(path: &str) -> Result<()> {
    // Read JSON from file
    let input = fs::read_to_string(path)?;

    // Parse JSON and render popup
    match parse_popup_json(&input).and_then(render_popup) {
        Ok(result) => {
            println!("{}", serde_json::to_string_pretty(&result)?);
            std::process::exit(0); // Success - user interaction completed
        }
        Err(e) => {
            let error = serde_json::json!({"error": e.to_string()});
            println!("{}", serde_json::to_string(&error)?);
            std::process::exit(1); // Error - parsing or rendering failed
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.stdin {
        // Read JSON from stdin and show popup
        run_stdin_mode()
    } else if let Some(file_path) = args.file {
        // Read JSON from file and show popup
        run_file_mode(&file_path)
    } else {
        // MCP server mode (default)
        let server_args = mcp_server::ServerArgs {
            include_only: args.include_only,
            exclude: args.exclude,
            list_templates: args.list_templates,
        };
        mcp_server::run(server_args)
    }
}
