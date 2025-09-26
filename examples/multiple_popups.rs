//! Example demonstrating multiple sequential popups from the same process

use popup_mcp::{render_popup, PopupDefinition, PopupResult};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define three different popups
    let popup1_def: PopupDefinition = serde_json::from_value(json!({
        "title": "User Settings",
        "elements": [
            {"type": "text", "content": "Configure your preferences:"},
            {"type": "checkbox", "label": "Enable notifications", "default": true},
            {"type": "slider", "label": "Volume", "min": 0, "max": 100, "default": 75},
            {"type": "multiselect", "label": "Theme", "options": ["Light", "Dark", "Auto"]}
        ]
    }))?;

    let popup2_def: PopupDefinition = serde_json::from_value(json!({
        "title": "Feedback Form",
        "elements": [
            {"type": "text", "content": "How was your experience?"},
            {"type": "multiselect", "label": "Rating", "options": ["Excellent", "Good", "Fair", "Poor"]},
            {"type": "textbox", "label": "Comments", "placeholder": "Optional feedback...", "rows": 3}
        ]
    }))?;

    let popup3_def: PopupDefinition = serde_json::from_value(json!({
        "title": "Quick Survey",
        "elements": [
            {"type": "text", "content": "Select all that apply:"},
            {"type": "multiselect", "label": "Interests", "options": ["Tech", "Sports", "Music", "Art", "Gaming"]},
            {"type": "checkbox", "label": "Subscribe to newsletter", "default": false}
        ]
    }))?;

    println!("Showing 3 popups sequentially...");
    println!("Close each popup to see the next one.");
    println!();

    // Show popups one at a time
    println!("Showing User Settings popup...");
    match render_popup(popup1_def) {
        Ok(result) => print_result("User Settings", result),
        Err(e) => println!("User Settings popup error: {}", e),
    }

    println!("\nShowing Feedback Form popup...");
    match render_popup(popup2_def) {
        Ok(result) => print_result("Feedback Form", result),
        Err(e) => println!("Feedback Form popup error: {}", e),
    }

    println!("\nShowing Quick Survey popup...");
    match render_popup(popup3_def) {
        Ok(result) => print_result("Quick Survey", result),
        Err(e) => println!("Quick Survey popup error: {}", e),
    }

    Ok(())
}

fn print_result(title: &str, result: PopupResult) {
    println!("\n{} Results:", title);
    println!("  Button: {}", result.button);
    if !result.values.is_empty() {
        println!("  Values:");
        for (key, value) in &result.values {
            println!("    {}: {}", key, value);
        }
    }
}
