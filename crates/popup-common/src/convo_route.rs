//! Deciding what the window does next, for a conversation.
//!
//! The judgment is compiled from the tree, so the authoring model writes
//! only what it would have written anyway: what it says, and what the
//! person might say back. Everything else here - the exits, the modal
//! questions, the ordered context, the destination preview - is structure
//! the tree already declared.
//!
//! The shape and the ordering are not guesses. They came out of 51
//! labelled cases over four deliberately varied trees (`jev-lab/`), where
//! this arrangement scored 50/51 with one wrong action. The parts that
//! earned their place, in the order they earned it:
//!
//! - **The lines offered, in order, with positions.** Dropping just this
//!   cost seven cases. It is what lets "the second one you listed" land.
//! - **One Choice over a small pool, with the modal moves as Nouls
//!   beside it.** The same moves crowded into one Choice measured worse.
//! - **A multiplicity Score, gated on its own confidence.** Per-line
//!   Nouls cannot tell "all of these" from one clear pick: an answer
//!   covering three lines scores each *lower* than a single clear pick
//!   scores its own.
//! - **One wide call.** Doubling the questions cost 3ms and changed no
//!   decision. A second call to break a tie, by contrast, manufactures
//!   confident answers on exactly the cases that should hand back.

use crate::convo::{Continues, Conversation, Line, Node};
use crate::route::{
    RouteAnswer, RoutePacket, ACTION_FLOOR, EXIT_NONE_FIT, EXIT_UNCLEAR, HOLD_FLOOR,
    HOLD_FLOOR_MANY, MANY_CONFIDENCE_FLOOR, MANY_FLOOR, NARROW_FLOOR, NOT_IMPLIED,
};
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::collections::HashMap;

/// Turns of history that travel with the judgment.
///
/// This is the context that actually pays. On twelve cases built to need
/// surrounding context it was worth two of them, more than any other
/// enrichment tried, because a reference like "do what worked on the auth
/// tests" points at something said earlier and at nothing else.
///
/// Two things measured *worse* and are deliberately not here. Numbering
/// the turns cost the gain back: a position helps for a list someone
/// points at by ordinal, and history is pointed at by content. And listing
/// the lines they turned down earlier lost two further cases, because
/// their text then competes with the lines actually on offer.
const HISTORY_KEPT: usize = 3;

/// Destination lines previewed per option, so the packet stays small
const PREVIEW_LINES: usize = 3;

/// One turn already taken, as the judgment sees it
#[derive(Debug, Clone, Serialize)]
pub struct Turn {
    #[serde(rename = "I said")]
    pub said: String,
    #[serde(rename = "they answered")]
    pub answered: String,
}

/// Everything the window knows when it asks what to do next
#[derive(Debug, Clone, Default)]
pub struct Standing {
    /// What they typed into this node, verbatim
    pub typed: Option<String>,
    /// Field label -> what they put in it
    pub fields: HashMap<String, Value>,
    /// Flags set so far, for the mechanical gates
    pub flags: HashMap<String, Value>,
    /// Turns already taken, oldest first
    pub history: Vec<Turn>,
}

/// What the window does next
#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    /// A gate held: take this line with no judgment at all
    Mechanical { line: usize },
    /// Lines need judging; send this
    Ask(RoutePacket),
    /// Nothing to leave by
    Done,
}

/// Decide, or build the packet that will decide.
pub fn next_step(convo: &Conversation, node: &Node, standing: &Standing) -> Step {
    let open = node.open_lines(&standing.flags);
    if open.is_empty() {
        return Step::Done;
    }
    // A single open line with a gate that held is a corridor the author
    // built on purpose; walking it costs nothing and asks nobody.
    if open.len() == 1 && open[0].1.when.is_some() {
        return Step::Mechanical { line: open[0].0 };
    }
    Step::Ask(packet(convo, node, standing))
}

/// Compile the judgment from the tree.
pub fn packet(convo: &Conversation, node: &Node, standing: &Standing) -> RoutePacket {
    let open = node.open_lines(&standing.flags);

    // The ordered context. Position is load-bearing: a reference like
    // "the second one" or "that last thing" has to have somewhere to land.
    let ways: Vec<Value> = open
        .iter()
        .enumerate()
        .map(|(shown, (idx, line))| {
            let mut entry = Map::new();
            entry.insert("position".into(), json!(shown + 1));
            entry.insert("they say".into(), json!(line.line));
            if let Some(means) = &line.means {
                entry.insert("which would mean".into(), json!(means));
            }
            entry.insert("weight".into(), json!(authored_weight(node, *idx)));
            match line.continues() {
                Continues::Hole => {
                    entry.insert("then".into(), json!("the conversation goes back to chat"));
                }
                Continues::Here(dest) => {
                    entry.insert("then I ask".into(), json!(preview(dest)));
                }
                Continues::At(id) => {
                    if let Some(dest) = convo.node(id) {
                        entry.insert("then I ask".into(), json!(preview(dest)));
                    }
                }
            }
            Value::Object(entry)
        })
        .collect();

    let mut state = Map::new();
    state.insert("what I just said".into(), json!(node.say));
    state.insert(
        "what they answered".into(),
        json!(standing.typed.as_deref().unwrap_or("")),
    );
    state.insert("the ways this could go, in order".into(), json!(ways));
    if !standing.fields.is_empty() {
        state.insert("what they have filled in".into(), json!(standing.fields));
    }
    if !standing.history.is_empty() {
        let recent: Vec<&Turn> = standing
            .history
            .iter()
            .rev()
            .take(HISTORY_KEPT)
            .rev()
            .collect();
        state.insert("how we got here".into(), json!(recent));
    }
    state.insert("what this conversation is for".into(), json!(convo.title));
    // The surrounding situation. Sent, never shown: the screen has to
    // stay a line or two, but the judgment places their words better the
    // more it knows about what is going on around them.
    if let Some(context) = &convo.context {
        state.insert("what is going on".into(), json!(context));
    }
    if let Some(aside) = &node.aside {
        state.insert("what this particular turn is about".into(), json!(aside));
    }

    // The pool the Choice runs over stays small on purpose.
    let mut criteria = Map::new();
    for (idx, line) in &open {
        let mut option = Map::new();
        option.insert("they are saying".into(), json!(line.criterion()));
        criteria.insert(key(*idx), Value::Object(option));
    }
    criteria.insert(
        EXIT_NONE_FIT.into(),
        json!({"they are saying": "something clear that is none of the lines above"}),
    );
    criteria.insert(
        EXIT_UNCLEAR.into(),
        json!({"they are saying": "too little to tell which line it is, or two of them equally"}),
    );

    let mut questions = Map::new();
    questions.insert(
        "action".into(),
        json!({
            "type": "choice",
            "instructions": {
                "question": "Which of these did the person just say?",
                "how to read": "`what they answered` is their own words and outranks anything they filled in. A reference like \"the second one\" or \"that last thing\" points into `the ways this could go, in order`; match it by position or by wording.",
                "each option": "is one of the ways I thought this could go, written as they might say it.",
                "how we got here": "background only; the decision is about what they just answered.",
            },
            "criteria": criteria,
        }),
    );

    // Per-line Nouls: a second path to action when the Choice is unsure,
    // and the way the set is read when more than one line applies.
    for (idx, line) in &open {
        questions.insert(
            format!("holds:{}", key(*idx)),
            json!({"type": "noul", "instructions":
                format!("Taken on its own, is this what the person is saying: {}?", line.criterion())}),
        );
    }

    questions.insert(
        "how_many".into(),
        json!({"type": "score",
               "instructions": "How many of the lines in `the ways this could go` is the person asking for? Several remarks about the same line still count as one.",
               "criteria": ["none of them", "one of them", "two of them", "three or more of them"]}),
    );

    for (name, question) in [
        ("undo", "Is the person saying what they filled in was a mistake and should simply be cleared, rather than pointing at a different line instead?"),
        ("narrow", "Has the person narrowed it to two of the lines but not to one?"),
        ("expand", "Is the person asking for something close to one of the lines but not quite any of them, so more choices here would serve better than moving on?"),
        ("confirm", "Has the person already said enough to answer what comes after the line they are taking?"),
    ] {
        questions.insert(
            format!("is:{}", name),
            json!({"type": "noul", "instructions": question}),
        );
    }

    questions.insert(
        "contradicts".into(),
        json!({"type": "noul", "instructions":
            "Does `what they answered` contradict or correct something in `what they have filled in`?"}),
    );

    // Lookahead. Never enough to skip a turn - measured at zero to one of
    // two questions - but enough to open the next one already answered.
    for (idx, line) in &open {
        let dest = match line.continues() {
            Continues::Here(d) => Some(d),
            Continues::At(id) => convo.node(id),
            Continues::Hole => None,
        };
        let Some(dest) = dest else { continue };
        if dest.lines.len() < 2 {
            continue;
        }
        let mut pool: Map<String, Value> = dest
            .lines
            .iter()
            .map(|l| (l.line.clone(), Value::Null))
            .collect();
        pool.insert(
            NOT_IMPLIED.into(),
            json!("nothing they have said implies any of them"),
        );
        questions.insert(
            format!("prefill:{}", key(*idx)),
            json!({
                "type": "choice",
                "instructions": format!(
                    "If they take \"{}\" and I then say \"{}\", which of the answers to that have they already given?",
                    line.line, dest.say),
                "criteria": pool,
            }),
        );
    }

    RoutePacket {
        state: Value::Object(state),
        questions: Value::Object(questions),
    }
}

/// Wire key for a line
pub fn key(index: usize) -> String {
    format!("line:l{}", index)
}

/// Index back out of a wire key
pub fn index_of(key: &str) -> Option<usize> {
    key.strip_prefix("line:l")?.parse().ok()
}

/// The author's prior. Order is the default, so a tree that never writes
/// `odds` still carries a weighting: first is likeliest.
pub fn authored_weight(node: &Node, index: usize) -> u32 {
    if let Some(odds) = node.lines[index].odds {
        return odds;
    }
    (node.lines.len() - index) as u32
}

fn preview(dest: &Node) -> Vec<&str> {
    dest.lines
        .iter()
        .take(PREVIEW_LINES)
        .map(|l| l.line.as_str())
        .collect()
}

/// What to do with the judgment, for a conversation.
///
/// The ordering is the measured one. Disowning comes first, because
/// acting on an answer the person just took back is the worst outcome
/// available and the previous design did exactly that. Multiplicity comes
/// before the exits, because an answer covering several lines reads as
/// `unclear` to a single Choice: not too little said, too much.
#[derive(Debug, Clone, PartialEq)]
pub enum Move {
    /// Take this line
    Take { line: usize, why: Why },
    /// Several lines hold: take the first, keep the rest
    Several { first: usize, rest: Vec<usize> },
    /// Clear what they filled in here and let them say it again
    Undo,
    /// Hand the conversation back to the assistant
    Back {
        reason: &'static str,
        leaning: Option<usize>,
    },
}

/// Which signal decided a `Take`
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Why {
    /// The Choice cleared the gate
    Matched,
    /// The Choice did not, but exactly one line held on its own
    HeldAlone,
    /// A gate held in code; nothing was judged
    Gate,
}

pub const REASON_NONE_FIT: &str = "none_fit";
pub const REASON_UNCLEAR: &str = "unclear";
pub const REASON_UNSURE: &str = "unsure";
pub const REASON_BETWEEN_TWO: &str = "between_two";
pub const REASON_ADJACENT: &str = "adjacent";
pub const REASON_UNAVAILABLE: &str = "unavailable";

pub fn decide(answer: &RouteAnswer, open: &[(usize, &Line)]) -> Move {
    let act = |name: &str| answer.actions.get(name).copied().unwrap_or(0.0);
    let held = |idx: usize| answer.holds.get(&key(idx)).copied().unwrap_or(0.0);

    if act("undo") >= ACTION_FLOOR {
        return Move::Undo;
    }

    if answer.how_many >= MANY_FLOOR && answer.how_many_confidence >= MANY_CONFIDENCE_FLOOR {
        let strong: Vec<usize> = open
            .iter()
            .map(|(i, _)| *i)
            .filter(|i| held(*i) >= HOLD_FLOOR_MANY)
            .collect();
        if let [first, rest @ ..] = strong.as_slice() {
            if !rest.is_empty() {
                return Move::Several {
                    first: *first,
                    rest: rest.to_vec(),
                };
            }
        }
    }

    if let Some(chosen) = answer.suggestion().and_then(index_of) {
        if open.iter().any(|(i, _)| *i == chosen) {
            return Move::Take {
                line: chosen,
                why: Why::Matched,
            };
        }
    }
    if answer.choice == EXIT_NONE_FIT || answer.choice == EXIT_UNCLEAR {
        return Move::Back {
            reason: if answer.choice == EXIT_UNCLEAR {
                REASON_UNCLEAR
            } else {
                REASON_NONE_FIT
            },
            leaning: None,
        };
    }

    let leaning = index_of(&answer.choice).filter(|i| open.iter().any(|(j, _)| j == i));
    if act("narrow") >= NARROW_FLOOR {
        return Move::Back {
            reason: REASON_BETWEEN_TWO,
            leaning,
        };
    }
    if act("expand") >= ACTION_FLOOR {
        return Move::Back {
            reason: REASON_ADJACENT,
            leaning,
        };
    }

    let strong: Vec<usize> = open
        .iter()
        .map(|(i, _)| *i)
        .filter(|i| held(*i) >= HOLD_FLOOR)
        .collect();
    match strong.as_slice() {
        [only] => Move::Take {
            line: *only,
            why: Why::HeldAlone,
        },
        [first, rest @ ..] if !rest.is_empty() => Move::Several {
            first: *first,
            rest: rest.to_vec(),
        },
        _ => Move::Back {
            reason: REASON_UNSURE,
            leaning,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn convo() -> Conversation {
        serde_json::from_value(json!({
        "title": "What should I do about the flaky test?",
        "start": "open",
        "nodes": {
          "open": {"say": "It fails about one run in five, always on CI.",
            "lines": [
              {"line": "Quarantine it and move on", "goto": "quarantine"},
              {"line": "Dig into why it's flaky",
               "means": "They want the cause found rather than the symptom hidden."},
              {"line": "Delete it, it wasn't earning its keep", "odds": 1}]},
          "quarantine": {"say": "Fine. I'll mark it skipped with a note.",
            "lines": [{"line": "Open an issue too"}, {"line": "No issue, just skip it"}]}
        }}))
        .unwrap()
    }

    fn answer(choice: &str, probs: &[(&str, f64)], conf: f64) -> RouteAnswer {
        RouteAnswer {
            choice: choice.to_string(),
            probabilities: probs.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            confidence: conf,
            holds: HashMap::new(),
            contradicts: 0.0,
            how_many: 1.0,
            how_many_confidence: 0.9,
            actions: HashMap::new(),
            prefill: HashMap::new(),
        }
    }

    #[test]
    fn the_lines_travel_in_order_with_somewhere_for_a_reference_to_land() {
        let c = convo();
        let node = c.node("open").unwrap();
        let p = packet(&c, node, &Standing::default());
        let ways = p.state["the ways this could go, in order"]
            .as_array()
            .unwrap();
        assert_eq!(ways[1]["position"], json!(2));
        assert_eq!(ways[1]["they say"], json!("Dig into why it's flaky"));
        // a line that says more than its label carries the longer reading
        assert!(ways[1]["which would mean"].is_string());
        // order is the prior, and `odds` overrides it
        assert_eq!(ways[0]["weight"], json!(3));
        assert_eq!(ways[2]["weight"], json!(1));
        // a branch is judged partly on where it leads
        assert_eq!(
            ways[0]["then I ask"],
            json!(["Open an issue too", "No issue, just skip it"])
        );
        // and a hole says so, rather than looking like a dead end
        assert_eq!(ways[2]["then"], json!("the conversation goes back to chat"));
    }

    #[test]
    fn the_pool_stays_small_and_the_modal_moves_ride_beside_it() {
        let c = convo();
        let p = packet(&c, c.node("open").unwrap(), &Standing::default());
        let q = p.questions.as_object().unwrap();
        let criteria = p.questions["action"]["criteria"].as_object().unwrap();
        assert_eq!(criteria.len(), 5); // three lines plus the two exits
        for modal in ["is:undo", "is:narrow", "is:expand", "is:confirm"] {
            assert_eq!(q[modal]["type"], json!("noul"));
        }
        assert_eq!(q["how_many"]["type"], json!("score"));
        assert!(q.contains_key("holds:line:l0"));
        // lookahead only where there is a next turn to look at
        assert!(q.contains_key("prefill:line:l0"));
        assert!(!q.contains_key("prefill:line:l2"));
    }

    #[test]
    fn a_gate_that_leaves_one_line_open_is_walked_without_asking() {
        let c: Conversation = serde_json::from_value(json!({
          "title": "t", "start": "n",
          "nodes": {"n": {"say": "s", "lines": [
            {"line": "only when we have keys", "when": "have_keys"},
            {"line": "otherwise", "when": "!have_keys"}]}}}))
        .unwrap();
        let standing = Standing {
            flags: HashMap::from([("have_keys".to_string(), json!(true))]),
            ..Default::default()
        };
        assert_eq!(
            next_step(&c, c.node("n").unwrap(), &standing),
            Step::Mechanical { line: 0 }
        );
    }

    #[test]
    fn taking_back_an_answer_beats_acting_on_it() {
        let c = convo();
        let node = c.node("open").unwrap();
        let open = node.open_lines(&HashMap::new());
        // the Choice is confident, but they just disowned what they said
        let mut a = answer("line:l0", &[("line:l0", 0.9)], 0.95);
        a.actions.insert("undo".into(), 0.96);
        assert_eq!(decide(&a, &open), Move::Undo);
    }

    #[test]
    fn several_lines_at_once_read_as_unclear_to_the_choice_alone() {
        let c = convo();
        let node = c.node("open").unwrap();
        let open = node.open_lines(&HashMap::new());
        let mut a = answer(EXIT_UNCLEAR, &[(EXIT_UNCLEAR, 0.5)], 0.6);
        a.holds.insert("line:l0".into(), 0.8);
        a.holds.insert("line:l1".into(), 0.7);
        // without the Score, this hands back
        assert!(matches!(decide(&a, &open), Move::Back { .. }));
        a.how_many = 2.4;
        a.how_many_confidence = 0.7;
        assert_eq!(
            decide(&a, &open),
            Move::Several {
                first: 0,
                rest: vec![1]
            }
        );
    }

    #[test]
    fn a_score_that_is_unsure_of_itself_is_not_read() {
        let c = convo();
        let node = c.node("open").unwrap();
        let open = node.open_lines(&HashMap::new());
        let mut a = answer(EXIT_UNCLEAR, &[(EXIT_UNCLEAR, 0.5)], 0.6);
        a.holds.insert("line:l0".into(), 0.8);
        a.holds.insert("line:l1".into(), 0.7);
        a.how_many = 1.6;
        a.how_many_confidence = 0.0; // the case that opened two branches
        assert!(matches!(decide(&a, &open), Move::Back { .. }));
    }

    #[test]
    fn one_line_holding_alone_acts_where_the_choice_stalled() {
        let c = convo();
        let node = c.node("open").unwrap();
        let open = node.open_lines(&HashMap::new());
        let mut a = answer("line:l1", &[("line:l1", 0.45), ("line:l0", 0.3)], 0.35);
        assert!(a.suggestion().is_none());
        a.holds.insert("line:l1".into(), 0.9);
        assert_eq!(
            decide(&a, &open),
            Move::Take {
                line: 1,
                why: Why::HeldAlone
            }
        );
    }
}
