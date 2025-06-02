use anyhow::Result;
use popup_mcp::{parse_popup_dsl, render_popup};
use std::io::{self, Read};

fn main() -> Result<()> {
    // Read DSL from stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    
    // Parse the DSL
    let definition = parse_popup_dsl(&input)?;
    
    // Render the popup and get results
    let result = render_popup(definition)?;
    
    // Output JSON to stdout
    println!("{}", serde_json::to_string_pretty(&result)?);
    
    Ok(())
}
