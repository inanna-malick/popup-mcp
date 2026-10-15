//! Pre-flight checks on a conversation, so authoring mistakes come back
//! as fixable errors instead of a window that behaves oddly.
//!
//! Two of these are not structural but editorial, and they are here on
//! purpose. A node that offers one line is not a branch, it is a prompt
//! with a single exit, and a tree of those is the wall of text this tool
//! exists to replace. A `say` that runs long is the same failure wearing
//! a different hat. Both are the contract the language is for, so both
//! fail loudly rather than degrading quietly.

use crate::convo::{Continues, Conversation, Field, Node, MAX_SAY, RESERVED_IDS};
use crate::validate::{suggest, ValidationIssue};
use std::collections::{HashMap, HashSet};

/// Fewest lines a node that continues may offer. One line is a corridor,
/// not a choice, and it externalizes no alternative future at all.
pub const MIN_LINES: usize = 2;

pub fn validate(convo: &Conversation) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    let named: Vec<&str> = convo.nodes.keys().map(String::as_str).collect();

    if convo.title.trim().is_empty() {
        issues.push(issue(
            "conversation",
            "needs a title saying what it decides",
        ));
    }
    if !convo.nodes.contains_key(&convo.start) {
        let hint = suggest(&convo.start, named.iter().copied())
            .map(|s| format!(" - did you mean \"{}\"?", s))
            .unwrap_or_default();
        issues.push(issue(
            "conversation",
            &format!("start names no node: \"{}\"{}", convo.start, hint),
        ));
    }

    let mut field_ids: HashMap<String, String> = HashMap::new();
    let mut flags: HashSet<String> = HashSet::new();
    for (_, node) in convo.walk() {
        flags.extend(node.set.keys().cloned());
        for line in &node.lines {
            flags.extend(line.set.keys().cloned());
        }
    }

    for (id, node) in convo.walk() {
        check_node(&id, node, &named, &mut field_ids, &flags, &mut issues);
    }

    for unreachable in unreachable_nodes(convo) {
        issues.push(issue(
            &format!("node \"{}\"", unreachable),
            "nothing reaches this node: no line goes here and it is not the start",
        ));
    }
    issues
}

fn check_node(
    id: &str,
    node: &Node,
    named: &[&str],
    field_ids: &mut HashMap<String, String>,
    flags: &HashSet<String>,
    issues: &mut Vec<ValidationIssue>,
) {
    let at = format!("node \"{}\"", id);

    if node.say.trim().is_empty() {
        issues.push(issue(
            &at,
            "say is empty: a node is a turn, so it has to say something",
        ));
    }
    if node.say.chars().count() > MAX_SAY {
        issues.push(issue(
            &at,
            &format!(
                "say is {} characters, over the {} limit. Put the detail in the lines the person \
                 can pick, not in a paragraph they have to read and then answer from scratch",
                node.say.chars().count(),
                MAX_SAY
            ),
        ));
    }
    if !node.lines.is_empty() && node.lines.len() < MIN_LINES {
        issues.push(issue(
            &at,
            "has one line. A node that continues needs at least two, and three or four is the \
             working range: the point is to lay out the ways this could go, not one happy path",
        ));
    }

    let mut known: Vec<String> = flags.iter().cloned().collect();
    for field in &node.ask {
        let fid = field.id();
        if RESERVED_IDS.contains(&fid) {
            issues.push(issue(
                &at,
                &format!("field id \"{}\" is reserved by the result", fid),
            ));
        }
        if let Some(seen) = field_ids.get(fid) {
            issues.push(issue(
                &at,
                &format!("field id \"{}\" is already used on node \"{}\"", fid, seen),
            ));
        } else {
            field_ids.insert(fid.to_string(), id.to_string());
        }
        if let Field::Number {
            min,
            max,
            step,
            default,
            ..
        } = field
        {
            if min >= max {
                issues.push(issue(
                    &at,
                    &format!("field \"{}\": min must be below max", fid),
                ));
            }
            if let Some(step) = step {
                if *step <= 0.0 {
                    issues.push(issue(
                        &at,
                        &format!("field \"{}\": step must be positive", fid),
                    ));
                }
            }
            if let Some(d) = default {
                if d < min || d > max {
                    issues.push(issue(
                        &at,
                        &format!("field \"{}\": default {} is outside min..max", fid, d),
                    ));
                }
            }
        }
        known.push(fid.to_string());
        check_when(&at, field.when(), &known, issues);
    }

    for (i, line) in node.lines.iter().enumerate() {
        let where_ = format!("{} line {}", at, i + 1);
        if line.line.trim().is_empty() {
            issues.push(issue(&where_, "line text is empty"));
        }
        if let Some(means) = &line.means {
            // The measured failure: a `means` that summarises the line's
            // intent upward instead of elaborating it loses the words the
            // judgment has to match against.
            if means.trim().is_empty() {
                issues.push(issue(&where_, "means is empty: drop it or write it out"));
            }
        }
        if line.then.is_some() && line.goto.is_some() {
            issues.push(issue(
                &where_,
                "has both then and goto: a line continues in one place, or nowhere at all",
            ));
        }
        if let Continues::At(target) = line.continues() {
            if !named.contains(&target) {
                let hint = suggest(target, named.iter().copied())
                    .map(|s| format!(" - did you mean \"{}\"?", s))
                    .unwrap_or_default();
                issues.push(issue(
                    &where_,
                    &format!("goto names no node: \"{}\"{}", target, hint),
                ));
            }
        }
        check_when(&where_, line.when.as_deref(), &known, issues);
    }
}

fn check_when(at: &str, when: Option<&str>, known: &[String], issues: &mut Vec<ValidationIssue>) {
    let Some(expr) = when else { return };
    match crate::parse_condition(expr) {
        Err(e) => issues.push(issue(at, &format!("when clause does not parse: {}", e))),
        Ok(parsed) => {
            let mut refs = Vec::new();
            crate::collect_refs(&parsed, &mut refs);
            for r in refs {
                if !known.iter().any(|k| k == &r) {
                    let hint = suggest(&r, known.iter().map(String::as_str))
                        .map(|s| format!(" - did you mean \"{}\"?", s))
                        .unwrap_or_default();
                    issues.push(issue(
                        at,
                        &format!(
                            "when clause refers to \"{}\", which is not a field or a flag \
                             anything sets{}",
                            r, hint
                        ),
                    ));
                }
            }
        }
    }
}

/// Named nodes nothing can reach. An inline `then` is reachable by
/// construction, so only the named ones can be stranded.
fn unreachable_nodes(convo: &Conversation) -> Vec<String> {
    let mut reached: HashSet<&str> = HashSet::new();
    for (_, node) in convo.walk() {
        for line in &node.lines {
            if let Continues::At(target) = line.continues() {
                reached.insert(target);
            }
        }
    }
    let mut out: Vec<String> = convo
        .nodes
        .keys()
        .filter(|id| id.as_str() != convo.start && !reached.contains(id.as_str()))
        .cloned()
        .collect();
    out.sort();
    out
}

fn issue(element: &str, message: &str) -> ValidationIssue {
    ValidationIssue {
        element: element.to_string(),
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn convo(v: serde_json::Value) -> Conversation {
        serde_json::from_value(v).unwrap()
    }

    fn good() -> serde_json::Value {
        json!({
        "title": "What should I do about the flaky test?",
        "start": "open",
        "nodes": {
          "open": {"say": "It fails about one run in five, always on CI.",
            "lines": [
              {"line": "Quarantine it and move on", "goto": "quarantine"},
              {"line": "Dig into why it's flaky"},
              {"line": "Delete it, it wasn't earning its keep"}]},
          "quarantine": {"say": "Fine. I'll mark it skipped with a note.",
            "lines": [{"line": "Open an issue too"}, {"line": "No issue, just skip it"}]}
        }})
    }

    #[test]
    fn a_well_formed_conversation_passes() {
        assert!(validate(&convo(good())).is_empty());
    }

    #[test]
    fn one_line_is_not_a_branch() {
        let mut v = good();
        v["nodes"]["open"]["lines"] = json!([{"line": "Sure, go ahead"}]);
        let issues = validate(&convo(v));
        assert!(issues.iter().any(|i| i.message.contains("one happy path")));
    }

    #[test]
    fn a_wall_of_text_is_rejected() {
        let mut v = good();
        v["nodes"]["open"]["say"] = json!("word ".repeat(200));
        let issues = validate(&convo(v));
        assert!(issues.iter().any(|i| i.message.contains("over the")));
    }

    #[test]
    fn a_goto_nowhere_suggests_the_node_you_meant() {
        let mut v = good();
        v["nodes"]["open"]["lines"][0]["goto"] = json!("quarantene");
        let issues = validate(&convo(v));
        assert!(issues
            .iter()
            .any(|i| i.message.contains("did you mean \"quarantine\"")));
    }

    #[test]
    fn a_stranded_node_is_an_error_not_dead_weight() {
        let mut v = good();
        v["nodes"]["orphan"] = json!({"say": "Nobody gets here.",
            "lines": [{"line": "a"}, {"line": "b"}]});
        let issues = validate(&convo(v));
        assert!(issues.iter().any(|i| i.element.contains("orphan")));
    }

    #[test]
    fn a_gate_must_name_something_that_exists() {
        let mut v = good();
        v["nodes"]["open"]["lines"][1]["when"] = json!("have_kesy");
        v["nodes"]["open"]["lines"][2]["set"] = json!({"have_keys": true});
        let issues = validate(&convo(v));
        assert!(issues
            .iter()
            .any(|i| i.message.contains("did you mean \"have_keys\"")));
    }

    #[test]
    fn a_flag_something_sets_is_a_legitimate_reference() {
        let mut v = good();
        v["nodes"]["open"]["lines"][0]["set"] = json!({"decided": true});
        v["nodes"]["quarantine"]["lines"][0]["when"] = json!("decided");
        assert!(validate(&convo(v)).is_empty());
    }

    #[test]
    fn a_line_cannot_continue_in_two_places() {
        let mut v = good();
        v["nodes"]["open"]["lines"][1]["goto"] = json!("quarantine");
        v["nodes"]["open"]["lines"][1]["then"] =
            json!({"say": "x", "lines": [{"line": "a"}, {"line": "b"}]});
        let issues = validate(&convo(v));
        assert!(issues.iter().any(|i| i.message.contains("one place")));
    }
}
