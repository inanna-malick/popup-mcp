use anyhow::Result;
use popup_mcp::{parse_popup_dsl, render_popup};
use std::io::{self, Read};

fn main() -> Result<()> {
    // Read DSL from stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    
    // Parse and render popup
    match parse_popup_dsl(&input).and_then(render_popup) {
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
