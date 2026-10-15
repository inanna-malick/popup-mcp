use anyhow::Result;
use clap::Parser;
use popup_gui::{mcp_server, parse_and_validate_popup_json, render_popup};
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

    /// Print the Jev packet a page would send, and exit. Development aid:
    /// the experiment lab diffs its own packet against this, so its
    /// findings cannot drift from the compiler the product actually runs.
    #[arg(long, requires = "file")]
    compile_packet: Option<String>,

    /// Answers for --compile-packet, as a JSON object of id -> value.
    /// Ids not on the named page are ignored.
    #[arg(long, value_name = "JSON")]
    answers: Option<String>,

    /// What the person typed, for --compile-packet on a conversation
    #[arg(long, value_name = "TEXT")]
    typed: Option<String>,

    /// Flags already set, for --compile-packet on a conversation
    #[arg(long, value_name = "JSON")]
    flags: Option<String>,

    /// Turns already taken, as [{"said","answered"}], for --compile-packet
    #[arg(long, value_name = "JSON")]
    history: Option<String>,
}

/// Compile and print one node's judgment, for a conversation. The lab
/// calls this rather than rebuilding the packet in Python, so an
/// experiment can never measure a packet the product does not send.
fn run_compile_node(path: &str, node_id: &str, args: &Args) -> Result<()> {
    use popup_common::convo::Conversation;
    use popup_common::convo_route::{packet, Standing, Turn};

    let convo: Conversation = serde_json::from_str(&fs::read_to_string(path)?)?;
    let issues = popup_common::convo_validate::validate(&convo);
    if !issues.is_empty() {
        let list: Vec<String> = issues.iter().map(ToString::to_string).collect();
        anyhow::bail!("invalid conversation:\n  {}", list.join("\n  "));
    }
    let node = convo
        .node(node_id)
        .ok_or_else(|| anyhow::anyhow!("no node with id \"{}\"", node_id))?;

    let parse =
        |raw: &Option<String>| -> Result<std::collections::HashMap<String, serde_json::Value>> {
            Ok(match raw {
                Some(text) => serde_json::from_str(text)?,
                None => Default::default(),
            })
        };
    let fields_by_id = parse(&args.answers)?;
    // Fields travel to the judgment by label, the way they were on screen
    let mut fields = std::collections::HashMap::new();
    for field in &node.ask {
        if let Some(value) = fields_by_id.get(field.id()) {
            fields.insert(field.label().to_string(), value.clone());
        }
    }
    let history: Vec<Turn> = match &args.history {
        Some(text) => serde_json::from_str::<Vec<serde_json::Value>>(text)?
            .into_iter()
            .map(|t| Turn {
                said: t["said"].as_str().unwrap_or_default().to_string(),
                answered: t["answered"].as_str().unwrap_or_default().to_string(),
            })
            .collect(),
        None => Vec::new(),
    };
    let standing = Standing {
        typed: args.typed.clone().filter(|t| !t.is_empty()),
        fields,
        flags: parse(&args.flags)?,
        history,
    };

    let p = packet(&convo, node, &standing);
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "state": p.state, "questions": p.questions
        }))?
    );
    Ok(())
}

/// Compile and print one page's routing packet without rendering anything
fn run_compile_packet(path: &str, page_id: &str, answers: Option<&str>) -> Result<()> {
    use popup_common::route;
    use popup_common::{find_element_by_id, Element};

    let definition = parse_and_validate_popup_json(&fs::read_to_string(path)?)?;
    let page = definition
        .elements
        .iter()
        .find(|e| e.page_id().as_deref() == Some(page_id))
        .ok_or_else(|| anyhow::anyhow!("no page with id \"{}\"", page_id))?;

    // Answers arrive keyed by id and travel to Jev keyed by label, the
    // same swap `route_nav::build_path` makes at run time
    let given: serde_json::Map<String, serde_json::Value> = match answers {
        Some(raw) => serde_json::from_str(raw)?,
        None => serde_json::Map::new(),
    };
    let mut labelled = std::collections::HashMap::new();
    for (id, value) in &given {
        if let Some(el) = find_element_by_id(&definition.elements, id) {
            // The same enrichment the GUI applies: an answer travels with
            // the description that was on screen beside it
            labelled.insert(
                el.label().to_string(),
                popup_gui::gui::route_nav::with_descriptions(el, value),
            );
        }
    }
    let path_so_far = vec![route::PageAnswers {
        page: match page {
            Element::Page { page, .. } => page.clone(),
            _ => String::new(),
        },
        answers: labelled,
        note: given
            .get("_note")
            .and_then(|v| v.as_str())
            .map(str::to_string),
    }];

    let packet = route::route_packet(&definition.title, &definition.elements, page, &path_so_far);
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "state": packet.state, "questions": packet.questions
        }))?
    );
    Ok(())
}

fn run_stdin_mode() -> Result<()> {
    // Read JSON from stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    if is_conversation(&input) {
        return run_conversation(&input);
    }

    // Parse, inject "Other" options, validate the render-ready definition, render
    match parse_and_validate_popup_json(&input).and_then(render_popup) {
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

/// Open a conversation and print how it ended
fn run_conversation(input: &str) -> Result<()> {
    use popup_common::convo::Conversation;

    let convo: Conversation = serde_json::from_str(input)?;
    let issues = popup_common::convo_validate::validate(&convo);
    if !issues.is_empty() {
        let list: Vec<String> = issues.iter().map(ToString::to_string).collect();
        let error =
            serde_json::json!({"error": format!("invalid conversation: {}", list.join("; "))});
        println!("{}", serde_json::to_string(&error)?);
        std::process::exit(1);
    }
    match popup_gui::convo_gui::run(convo) {
        Ok(ending) => {
            println!("{}", serde_json::to_string_pretty(&ending)?);
            std::process::exit(0);
        }
        Err(e) => {
            let error = serde_json::json!({"error": e.to_string()});
            println!("{}", serde_json::to_string(&error)?);
            std::process::exit(1);
        }
    }
}

/// A conversation names its nodes; the older element schema has elements.
/// Tell them apart by shape rather than by a flag.
fn is_conversation(input: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(input)
        .map(|v| v.get("nodes").is_some())
        .unwrap_or(false)
}

fn run_file_mode(path: &str) -> Result<()> {
    // Read JSON from file
    let input = fs::read_to_string(path)?;
    if is_conversation(&input) {
        return run_conversation(&input);
    }

    // Parse, inject "Other" options, validate the render-ready definition, render
    match parse_and_validate_popup_json(&input).and_then(render_popup) {
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

    if let (Some(node), Some(file)) = (args.compile_packet.as_deref(), args.file.as_deref()) {
        // A conversation names its nodes; the older element schema has
        // pages. Tell them apart by shape rather than by a flag.
        let raw = fs::read_to_string(file)?;
        return if is_conversation(&raw) {
            run_compile_node(file, node, &args)
        } else {
            run_compile_packet(file, node, args.answers.as_deref())
        };
    }

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
