use popup_mcp::dsl::parse_popup_dsl;

fn main() {
    println!("Testing enhanced error messages...\n");

    // Test 1: Missing quotes
    println!("Test 1: Missing quotes around title");
    println!("Input: popup Title []");
    match parse_popup_dsl("popup Title []") {
        Err(e) => println!("Error:\n{}\n", e),
        Ok(_) => println!("Unexpectedly succeeded!\n"),
    }

    // Test 2: Invalid widget
    println!("Test 2: Invalid widget in simplified syntax");
    println!("Input: [Title: invalid_widget \"test\"]");
    match parse_popup_dsl("[Title: invalid_widget \"test\"]") {
        Err(e) => println!("Error:\n{}\n", e),
        Ok(_) => println!("Unexpectedly succeeded!\n"),
    }

    // Test 3: Empty popup
    println!("Test 3: Empty popup body");
    println!("Input: popup \"Bad\" [");
    match parse_popup_dsl("popup \"Bad\" [") {
        Err(e) => println!("Error:\n{}\n", e),
        Ok(_) => println!("Unexpectedly succeeded!\n"),
    }

    // Test 4: Format mixing
    println!("Test 4: Format mixing (classic with bare text)");
    println!("Input: popup \"Title\" [ Name: [textbox] ]");
    match parse_popup_dsl("popup \"Title\" [ Name: [textbox] ]") {
        Err(e) => println!("Error:\n{}\n", e),
        Ok(_) => println!("Unexpectedly succeeded!\n"),
    }

    // Test 5: Simplified syntax with wrong structure
    println!("Test 5: Simplified syntax with wrong structure");
    println!("Input: [SPIKE: Quick Check\\n  Thing 1: [textbox]\\n]");
    match parse_popup_dsl("[SPIKE: Quick Check\n  Thing 1: [textbox]\n]") {
        Err(e) => println!("Error:\n{}", e),
        Ok(_) => println!("Unexpectedly succeeded!"),
    }
}