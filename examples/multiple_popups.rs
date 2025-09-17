//! Example demonstrating multiple concurrent popups from the same process

use popup_mcp::{render_popup_sequential, PopupDefinition, PopupResult};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define three different popups
    let popup1_def: PopupDefinition = serde_json::from_value(json!({
        "title": "User Settings",
        "elements": [
            {"type": "text", "content": "Configure your preferences:"},
            {"type": "checkbox", "label": "Enable notifications", "default": true},
            {"type": "slider", "label": "Volume", "min": 0, "max": 100, "default": 75},
            {"type": "choice", "label": "Theme", "options": ["Light", "Dark", "Auto"]}
        ]
    }))?;

    let popup2_def: PopupDefinition = serde_json::from_value(json!({
        "title": "Feedback Form",
        "elements": [
            {"type": "text", "content": "How was your experience?"},
            {"type": "choice", "label": "Rating", "options": ["Excellent", "Good", "Fair", "Poor"]},
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

    println!("Spawning 3 popups...");
    println!("Due to GUI limitations, popups will appear one at a time in sequence.");
    println!("Close each popup to see the next one.");
    println!();

    // Spawn all three popups (they will show sequentially)
    let handle1 = render_popup_sequential(popup1_def);
    let handle2 = render_popup_sequential(popup2_def);
    let handle3 = render_popup_sequential(popup3_def);

    // Wait for all popups to complete
    let (result1, result2, result3) = tokio::join!(handle1, handle2, handle3);

    // Process results
    match result1 {
        Ok(result) => print_result("User Settings", result),
        Err(e) => println!("User Settings popup error: {}", e),
    }

    match result2 {
        Ok(result) => print_result("Feedback Form", result),
        Err(e) => println!("Feedback Form popup error: {}", e),
    }

    match result3 {
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
