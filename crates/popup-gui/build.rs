fn main() {
    // This ensures the pest grammar is recompiled when changed
    println!("cargo:rerun-if-changed=src/popup.pest");
}
