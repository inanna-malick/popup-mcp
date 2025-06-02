use anyhow::Result;
use popup_mcp::{parse_popup_dsl, render_popup};
use std::io::{self, Read};

fn main() -> Result<()> {
    // Wrap everything to ensure we always output valid JSON
    let result = (|| -> Result<_> {
        // Read DSL from stdin
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        
        // Parse the DSL
        let definition = parse_popup_dsl(&input)?;
        
        // Render the popup and get results
        render_popup(definition)
    })();
    
    // Always output valid JSON, even on error
    match result {
        Ok(popup_result) => {
            println!("{}", serde_json::to_string_pretty(&popup_result)?);
        }
        Err(e) => {
            // Output error as JSON so MCP server can parse it
            let error_json = serde_json::json!({
                "error": format!("Failed to render popup: {}", e)
            });
            println!("{}", serde_json::to_string(&error_json)?);
        }
    }
    
    Ok(())
}
