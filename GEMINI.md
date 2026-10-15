# Gemini Code Assistant Context

This document provides context for the Gemini Code Assistant to understand and effectively assist with the development of the `popup-mcp` project.

## Project Overview

The primary implementation is now the Cloudflare MCP App in `apps/popup`, using TypeScript and Solid. Start with `README.md` and `apps/popup/README.md` for connection, authoring, testing, and deployment. The sections below describe the native Rust implementation.

`popup-mcp` is a Rust-based tool that enables the creation of native graphical user interface (GUI) popups from a JSON definition. These popups can be used to create interactive dialogs, forms, and conditional workflows. The project can be used as a command-line tool to display popups from a file or standard input, or it can run as a Model Context Protocol (MCP) server to allow AI assistants to trigger and interact with these popups.

The project is structured as a Cargo workspace with two main crates:

*   `popup-common`: This crate defines the core data structures for the popup definitions, including the various UI elements (`Element`), the popup state (`PopupState`), and the result of a popup interaction (`PopupResult`). It uses `serde` for serialization and deserialization of the JSON data and `schemars` to generate a JSON schema for the popup definitions.
*   `popup-gui`: This crate contains the main application logic. It uses the `egui` GUI library to render the popups. It also includes the command-line interface (using `clap`) and the MCP server implementation.

## Building and Running

The project is built and managed using Cargo.

### Building

To build the project, run the following command from the root of the repository:

```bash
cargo build --release
```

### Running

The `popup-gui` executable can be run in several modes:

*   **From a file:**

    ```bash
    cargo run --release --bin popup -- --file examples/user_profile.json
    ```

*   **From stdin:**

    ```bash
    cat examples/simple_confirm.json | cargo run --release --bin popup -- --stdin
    ```

*   **As an MCP server:**

    ```bash
    cargo run --release --bin popup
    ```

### Testing

To run the tests for the entire workspace, use the following command:

```bash
cargo test
```

## Development Conventions

*   **Code Style:** The project follows standard Rust conventions and is formatted using `rustfmt`.
*   **Error Handling:** The project uses the `anyhow` crate for error handling.
*   **GUI:** The GUI is built using the `egui` crate. The UI is defined declaratively based on the `PopupDefinition` struct.
*   **JSON Schema:** The JSON schema for the popup definitions is defined in `crates/popup-common/src/schema.rs` and is generated using the `schemars` crate.
*   **Conditional Logic:** Popups can have conditional logic to show or hide elements based on the user's input. This is handled by the `when` field in the `Element` struct and the `evaluate_condition` function in `crates/popup-common/src/condition.rs`.
*   **Templates:** The MCP server supports templates to create reusable popup definitions. Templates are defined in the `.claude/templates` directory.
