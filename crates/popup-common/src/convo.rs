//! A conversation as a partial tree of futures.
//!
//! One tool call is a single application of a coalgebra: the authoring
//! model unfolds `seed -> Node (Free Node seed)`, several levels deep
//! where it is confident and one level where it is not, leaving holes
//! wherever the unfolding should come back to it. The runtime walks the
//! result with the person; at a hole it hands back with the path taken.
//!
//! A node is what the assistant says. A line is what the person might say
//! next, written in their voice. A line's `line` text does three jobs at
//! once: the button they see, the thing their free text is matched
//! against, and the explanation shown when that branch is taken. Keeping
//! them one string is the point - the previous design held the same
//! meaning in an option label and again in a routing condition, and when
//! the two drifted apart routing collapsed.
//!
//! Likelihood is authored, not only measured. The order of `lines` is the
//! prior; `odds` overrides it where order is not enough. That is what
//! makes the depth of an unfold a decision rather than an accident: deep
//! where the mass is, one bare line where it is not.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Keys the result reserves for itself
pub const RESERVED_IDS: &[&str] = &["status", "path", "fields", "flags", "error", "said"];

/// A line with no continuation hands the conversation back. Recorded in
/// the path under this name so the caller can see where it stopped.
pub const HOLE: &str = "back_to_you";

/// Longest `say` a node may carry. A wall of text is the thing this tool
/// exists to avoid: reading prose serially and composing a reply from
/// nothing is exactly the work the lines are meant to do in advance.
pub const MAX_SAY: usize = 400;

/// A conversation: a start node and the nodes reachable from it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Conversation {
    /// What this conversation decides, in one sentence
    pub title: String,
    /// Id of the node to open with
    pub start: String,
    /// What is going on around this conversation. Never rendered: the
    /// screen and the judgment have different budgets.
    ///
    /// Use sparingly. Measured twice and it changed nothing either time,
    /// while padding the state measurably hurts: on twelve cases needing
    /// context, the turns already taken were worth two cases on their own,
    /// and adding everything else on top gave those two back. Narration
    /// about the situation is not what a person's words point at. Write
    /// this only for something a line genuinely cannot carry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// Named nodes. Inline `then` subtrees do not appear here; only nodes
    /// that something can `goto`.
    pub nodes: HashMap<String, Node>,
}

/// One turn: what the assistant says, and what the person might say back.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    /// The assistant's turn. A line or two, never a wall.
    pub say: String,
    /// Precision the node needs that no line can carry: a number, a date,
    /// a name. Not a branch - anything enumerable is a line.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ask: Vec<Field>,
    /// What the person might say next, most likely first. Two minimum on
    /// a node that continues; three or four is the working range.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lines: Vec<Line>,
    /// Context for this turn alone, sent and never shown: what makes
    /// these particular lines the ones worth offering here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aside: Option<String>,
    /// Flags set on arriving here
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub set: HashMap<String, Value>,
}

/// One thing the person might say, and where it leads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Line {
    /// In the person's own voice, as they might actually say it
    pub line: String,
    /// A longer reading of the same stance, for when the short line
    /// understates it. Must carry the line's own vocabulary rather than
    /// abstracting upward into a summary of intent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub means: Option<String>,
    /// Relative weight. Order is the default prior; this overrides it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub odds: Option<u32>,
    /// Mechanical gate over flags and answered fields. Checked in code,
    /// costs nothing, and never reaches the judgment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,
    /// Flags set when this line is taken
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub set: HashMap<String, Value>,
    /// The conversation continues here, inline
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub then: Option<Box<Node>>,
    /// The conversation continues at a named node: a hub, or a loop back
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goto: Option<String>,
}

/// Where a line goes. Absent is not an error: it is the hole that makes an
/// unlikely future cost one sentence to write down.
#[derive(Debug, Clone, PartialEq)]
pub enum Continues<'a> {
    /// More tree, written inline
    Here(&'a Node),
    /// A named node
    At(&'a str),
    /// Nothing authored past this point: hand back
    Hole,
}

impl Line {
    pub fn continues(&self) -> Continues<'_> {
        match (&self.then, &self.goto) {
            (Some(node), _) => Continues::Here(node),
            (None, Some(id)) => Continues::At(id),
            (None, None) => Continues::Hole,
        }
    }

    /// What the judgment matches against: the longer reading when there is
    /// one, else the line itself
    pub fn criterion(&self) -> &str {
        self.means.as_deref().unwrap_or(&self.line)
    }

    /// Stable handle for this line within its node
    pub fn id(&self, index: usize) -> String {
        format!("l{}", index)
    }
}

/// Precision a node asks for, where no line could carry it
/// The kind is the JSON key and its value is the label, so a field reads
/// as one line. Deserialization is strict, with a suggestion on a typo:
/// an unknown key must be an error, not a silently ignored instruction.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Field {
    /// Free text: a name, a key, a sentence
    Text {
        text: String,
        id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        placeholder: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        default: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        rows: Option<usize>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<String>,
    },
    /// A quantity. Arithmetic stays in code; the judgment never infers one.
    Number {
        number: String,
        id: String,
        min: f64,
        max: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        step: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        default: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<String>,
    },
}

const TEXT_KEYS: &[&str] = &["text", "id", "placeholder", "default", "rows", "when"];
const NUMBER_KEYS: &[&str] = &["number", "id", "min", "max", "step", "default", "when"];

impl<'de> Deserialize<'de> for Field {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::Error as _;
        let map = serde_json::Map::deserialize(d)?;
        let (kind, allowed) = if map.contains_key("text") {
            ("text", TEXT_KEYS)
        } else if map.contains_key("number") {
            ("number", NUMBER_KEYS)
        } else {
            return Err(D::Error::custom(
                "a field needs \"text\" or \"number\" as its first key, carrying the label",
            ));
        };
        for key in map.keys() {
            if !allowed.contains(&key.as_str()) {
                let hint = crate::validate::suggest(key, allowed.iter().copied())
                    .map(|s| format!(" - did you mean \"{}\"?", s))
                    .unwrap_or_default();
                return Err(D::Error::custom(format!(
                    "{} field: unknown key \"{}\"{}. Allowed: {}",
                    kind,
                    key,
                    hint,
                    allowed.join(", ")
                )));
            }
        }
        let value = Value::Object(map);
        if kind == "text" {
            serde_json::from_value::<TextField>(value)
                .map(Field::from)
                .map_err(D::Error::custom)
        } else {
            serde_json::from_value::<NumberField>(value)
                .map(Field::from)
                .map_err(D::Error::custom)
        }
    }
}

#[derive(Deserialize)]
struct TextField {
    text: String,
    id: String,
    placeholder: Option<String>,
    default: Option<String>,
    rows: Option<usize>,
    when: Option<String>,
}

#[derive(Deserialize)]
struct NumberField {
    number: String,
    id: String,
    min: f64,
    max: f64,
    step: Option<f64>,
    default: Option<f64>,
    when: Option<String>,
}

impl From<TextField> for Field {
    fn from(f: TextField) -> Self {
        Field::Text {
            text: f.text,
            id: f.id,
            placeholder: f.placeholder,
            default: f.default,
            rows: f.rows,
            when: f.when,
        }
    }
}

impl From<NumberField> for Field {
    fn from(f: NumberField) -> Self {
        Field::Number {
            number: f.number,
            id: f.id,
            min: f.min,
            max: f.max,
            step: f.step,
            default: f.default,
            when: f.when,
        }
    }
}

impl Field {
    pub fn id(&self) -> &str {
        match self {
            Field::Text { id, .. } | Field::Number { id, .. } => id,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Field::Text { text, .. } => text,
            Field::Number { number, .. } => number,
        }
    }

    pub fn when(&self) -> Option<&str> {
        match self {
            Field::Text { when, .. } | Field::Number { when, .. } => when.as_deref(),
        }
    }
}

impl Node {
    /// A node nobody can leave: the conversation ends here
    pub fn is_terminal(&self) -> bool {
        self.lines.is_empty()
    }

    /// Lines whose gate currently holds, paired with their index
    pub fn open_lines<'a>(&'a self, values: &HashMap<String, Value>) -> Vec<(usize, &'a Line)> {
        self.lines
            .iter()
            .enumerate()
            .filter(|(_, l)| crate::when_is_visible(l.when.as_deref(), values))
            .collect()
    }
}

impl Conversation {
    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// Every node in the conversation, named ones and inline subtrees
    /// alike, as (id, node). An inline node's id is its parent's id plus
    /// the line that reached it, so the path can name it.
    pub fn walk(&self) -> Vec<(String, &Node)> {
        let mut out = Vec::new();
        for (id, node) in &self.nodes {
            collect(id.clone(), node, &mut out);
        }
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }
}

fn collect<'a>(id: String, node: &'a Node, out: &mut Vec<(String, &'a Node)>) {
    out.push((id.clone(), node));
    for (i, line) in node.lines.iter().enumerate() {
        if let Some(child) = &line.then {
            collect(format!("{}/{}", id, line.id(i)), child, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn convo() -> Conversation {
        serde_json::from_value(json!({
          "title": "How should I handle the failing payment tests?",
          "start": "open",
          "nodes": {
            "open": {
              "say": "Four tests fail after the Stripe bump. The fixtures drifted.",
              "lines": [
                {"line": "Just patch the fixtures, I want this off my plate",
                 "means": "They want the fastest thing that makes CI green and accept the drift staying.",
                 "then": {"say": "Fair. That leaves the drift in place.",
                          "lines": [{"line": "Add a test that catches it next time"},
                                    {"line": "Leave it, I'll deal with it when it bites"}]}},
                {"line": "Regenerate them from the sandbox properly", "goto": "regen"},
                {"line": "Neither, the bump itself was a mistake", "odds": 1}
              ]},
            "regen": {
              "say": "That takes about an hour and needs sandbox keys.",
              "ask": [{"text": "Sandbox key", "id": "key"}],
              "lines": [
                {"line": "Go ahead", "set": {"committed": true}},
                {"line": "Too slow, back to patching", "goto": "open"}
              ]}
          }
        }))
        .unwrap()
    }

    #[test]
    fn a_line_without_a_continuation_is_a_hole() {
        let c = convo();
        let open = c.node("open").unwrap();
        // authored deep where the mass is, bare where it is not
        assert!(matches!(open.lines[0].continues(), Continues::Here(_)));
        assert_eq!(open.lines[1].continues(), Continues::At("regen"));
        assert_eq!(open.lines[2].continues(), Continues::Hole);
    }

    #[test]
    fn the_criterion_is_the_line_unless_it_understates() {
        let c = convo();
        let open = c.node("open").unwrap();
        assert!(open.lines[0]
            .criterion()
            .starts_with("They want the fastest"));
        // no `means`, so the line speaks for itself and cannot drift from
        // a second copy of its own meaning
        assert_eq!(
            open.lines[1].criterion(),
            "Regenerate them from the sandbox properly"
        );
    }

    #[test]
    fn walking_names_inline_nodes_by_the_line_that_reached_them() {
        let c = convo();
        let ids: Vec<String> = c.walk().into_iter().map(|(id, _)| id).collect();
        assert!(ids.contains(&"open".to_string()));
        assert!(ids.contains(&"open/l0".to_string()));
        assert!(ids.contains(&"regen".to_string()));
    }

    #[test]
    fn a_gate_closes_a_line_without_any_judgment() {
        let node: Node = serde_json::from_value(json!({
            "say": "Where should it run?",
            "lines": [
                {"line": "On the server we already have", "when": "have_server"},
                {"line": "Somewhere managed"}
            ]
        }))
        .unwrap();
        let none = HashMap::new();
        assert_eq!(node.open_lines(&none).len(), 1);
        let yes = HashMap::from([("have_server".to_string(), json!(true))]);
        assert_eq!(node.open_lines(&yes).len(), 2);
    }

    #[test]
    fn unknown_keys_are_rejected_rather_than_ignored() {
        let bad = serde_json::from_value::<Node>(json!({
            "say": "x", "linez": [{"line": "y"}]
        }));
        assert!(bad.is_err());
    }
}
