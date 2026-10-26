//! Walking a conversation, and the record it leaves.
//!
//! The walk only goes forward. A line taken opens what it leads to; a
//! line with nothing past it hands back. `goto` returns to a hub, which
//! is forward motion that revisits a node, not a rewind - there is no
//! Back, because unsaying something is not a thing conversations do.
//!
//! The record is the point of the whole construction. Each hop keeps what
//! was said, which line was taken, **what weight I had given it before
//! they spoke**, and the expectations that lost. A rejection of a line I
//! weighted low carries several times the information of a confirmation,
//! and it arrives with the correction in their own words. That is what is
//! worth keeping after the window closes.

use crate::convo::{Continues, Conversation, Field, Node, HOLE};
use crate::convo_route::{Move, Turn, Why};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;

/// An expectation that did not win, with what I thought of it beforehand
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Rejected {
    /// The line, verbatim, as it was offered
    pub line: String,
    /// The weight I gave it before they said anything
    pub odds: u32,
    /// What the judgment put on it, when there was one
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mass: Option<f64>,
}

/// One turn of the walk
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Hop {
    /// Node this turn happened at
    pub at: String,
    /// What I said
    pub said: String,
    /// What they typed, when they typed rather than picked
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answered: Option<String>,
    /// The line taken, verbatim
    #[serde(skip_serializing_if = "Option::is_none")]
    pub took: Option<String>,
    /// The weight I had given that line beforehand
    #[serde(skip_serializing_if = "Option::is_none")]
    pub odds: Option<u32>,
    /// picked | matched | held | gate | queued | none
    pub how: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    /// Lines that lost, each with the weight I had given it
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rejected: Vec<Rejected>,
    /// Other lines that also held, taken after this one
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub also: Vec<String>,
    /// Where it went: a node id, or `back_to_you`
    pub to: String,
    /// Why it handed back, when it did
    #[serde(skip_serializing_if = "Option::is_none")]
    pub because: Option<&'static str>,
}

/// How the conversation ended
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Ending {
    /// Ran out of authored tree: the assistant picks it up from here
    BackToYou {
        /// none_fit, unclear, unsure, between_two, adjacent, unavailable,
        /// or `hole` when the author simply stopped there
        because: String,
        path: Vec<Hop>,
        fields: HashMap<String, Value>,
        flags: HashMap<String, Value>,
    },
    /// Reached a node with nothing to leave by
    Done {
        path: Vec<Hop>,
        fields: HashMap<String, Value>,
        flags: HashMap<String, Value>,
    },
    /// Escape
    Cancelled,
}

/// Reason recorded when a line simply had nothing authored past it. Not a
/// failure: it is where the unfolding was always going to come back.
pub const BECAUSE_HOLE: &str = "hole";

/// Where the walk is. Named nodes are their id; a node written inline is
/// addressed by the line that reached it, so the record can name it.
pub type Spot = String;

/// Resolve a spot to its node
pub fn at<'a>(convo: &'a Conversation, spot: &str) -> Option<&'a Node> {
    let (head, rest) = match spot.split_once('/') {
        Some((h, r)) => (h, Some(r)),
        None => (spot, None),
    };
    let mut node = convo.node(head)?;
    let Some(rest) = rest else { return Some(node) };
    for step in rest.split('/') {
        let index: usize = step.strip_prefix('l')?.parse().ok()?;
        node = node.lines.get(index)?.then.as_deref()?;
    }
    Some(node)
}

pub struct Walk<'a> {
    convo: &'a Conversation,
    spot: Spot,
    /// Lines that also held, waiting their turn. Forward-only: the queue
    /// is just more forward.
    queue: Vec<Spot>,
    flags: HashMap<String, Value>,
    fields: HashMap<String, Value>,
    path: Vec<Hop>,
    history: Vec<Turn>,
    ending: Option<Ending>,
}

impl<'a> Walk<'a> {
    pub fn new(convo: &'a Conversation) -> Self {
        let mut walk = Self {
            convo,
            spot: convo.start.clone(),
            queue: Vec::new(),
            flags: HashMap::new(),
            fields: HashMap::new(),
            path: Vec::new(),
            history: Vec::new(),
            ending: None,
        };
        walk.arrive();
        walk
    }

    pub fn spot(&self) -> &str {
        &self.spot
    }

    pub fn node(&self) -> &'a Node {
        at(self.convo, &self.spot).expect("validated conversation")
    }

    pub fn flags(&self) -> &HashMap<String, Value> {
        &self.flags
    }

    pub fn fields(&self) -> &HashMap<String, Value> {
        &self.fields
    }

    pub fn history(&self) -> &[Turn] {
        &self.history
    }

    pub fn ending(&self) -> Option<&Ending> {
        self.ending.as_ref()
    }

    /// Record a field answer on the current node
    pub fn fill(&mut self, id: &str, value: Value) {
        self.fields.insert(id.to_string(), value);
    }

    /// Fields on the current node whose gate holds
    pub fn open_fields(&self) -> Vec<&'a Field> {
        self.node()
            .ask
            .iter()
            .filter(|f| crate::when_is_visible(f.when(), &self.gate_values()))
            .collect()
    }

    /// What the mechanical gates see: flags and answered fields together
    pub fn gate_values(&self) -> HashMap<String, Value> {
        let mut values = self.flags.clone();
        values.extend(self.fields.clone());
        values
    }

    /// They picked a line outright. Nothing is judged: a click is already
    /// the answer, which is why the judgment only ever sees typed text.
    pub fn pick(&mut self, index: usize) {
        self.take(index, None, "picked", None, Vec::new(), Vec::new());
    }

    /// Act on a judgment.
    pub fn apply(&mut self, decision: Move, typed: Option<String>, evidence: &Evidence) {
        match decision {
            Move::Take { line, why } => {
                let how = match why {
                    Why::Matched => "matched",
                    Why::HeldAlone => "held",
                    Why::Gate => "gate",
                };
                let rejected = self.losers(&[line], evidence);
                self.take(line, typed, how, evidence.confidence, rejected, Vec::new());
            }
            Move::Several { first, rest } => {
                let mut taken = vec![first];
                taken.extend(rest.iter().copied());
                let rejected = self.losers(&taken, evidence);
                let also: Vec<String> = rest
                    .iter()
                    .filter_map(|i| self.node().lines.get(*i).map(|l| l.line.clone()))
                    .collect();
                // The rest are queued before the first is taken, so that
                // a first line with nothing past it resumes into them
                // rather than ending the conversation early.
                let here = self.spot.clone();
                let queued: Vec<Spot> = rest
                    .iter()
                    .filter_map(|i| self.destination(&here, *i))
                    .collect();
                self.queue.extend(queued);
                self.take(first, typed, "matched", evidence.confidence, rejected, also);
            }
            Move::Undo => self.undo(typed),
            Move::Back { reason, leaning } => {
                let rejected = self.losers(&[], evidence);
                let took = leaning.and_then(|i| self.node().lines.get(i));
                self.path.push(Hop {
                    at: self.spot.clone(),
                    said: self.node().say.clone(),
                    answered: typed,
                    took: took.map(|l| format!("leaning: {}", l.line)),
                    odds: None,
                    how: "none",
                    confidence: evidence.confidence,
                    rejected,
                    also: Vec::new(),
                    to: HOLE.to_string(),
                    because: Some(reason),
                });
                self.hand_back(reason);
            }
        }
    }

    /// They took back what they filled in here. Clearing beats routing on
    /// a mis-click, which is what an edge-only pool did with this answer.
    pub fn undo(&mut self, typed: Option<String>) {
        let ids: Vec<String> = self.node().ask.iter().map(|f| f.id().to_string()).collect();
        for id in &ids {
            self.fields.remove(id);
        }
        self.path.push(Hop {
            at: self.spot.clone(),
            said: self.node().say.clone(),
            answered: typed,
            took: None,
            odds: None,
            how: "none",
            confidence: None,
            rejected: Vec::new(),
            also: Vec::new(),
            to: self.spot.clone(),
            because: Some("undo"),
        });
    }

    /// End it here, whatever the reason
    pub fn hand_back(&mut self, reason: &'static str) {
        self.ending = Some(Ending::BackToYou {
            because: reason.to_string(),
            path: std::mem::take(&mut self.path),
            fields: self.fields.clone(),
            flags: self.flags.clone(),
        });
    }

    pub fn cancel(&mut self) {
        self.ending = Some(Ending::Cancelled);
    }

    // --- internals ---------------------------------------------------

    fn take(
        &mut self,
        index: usize,
        typed: Option<String>,
        how: &'static str,
        confidence: Option<f64>,
        rejected: Vec<Rejected>,
        also: Vec<String>,
    ) {
        let node = self.node();
        let Some(line) = node.lines.get(index) else {
            return;
        };
        let said = node.say.clone();
        let text = line.line.clone();
        let odds = crate::convo_route::authored_weight(node, index);
        let sets = line.set.clone();
        let here = self.spot.clone();
        let destination = self.destination(&here, index);

        self.flags.extend(sets);
        self.history.push(Turn {
            said: said.clone(),
            answered: typed.clone().unwrap_or_else(|| text.clone()),
        });
        self.path.push(Hop {
            at: here,
            said,
            answered: typed,
            took: Some(text),
            odds: Some(odds),
            how,
            confidence,
            rejected,
            also,
            to: destination.clone().unwrap_or_else(|| HOLE.to_string()),
            because: destination.is_none().then_some(BECAUSE_HOLE),
        });

        match destination {
            Some(spot) => {
                self.spot = spot;
                self.arrive();
            }
            // Nothing authored past this line. That is where the
            // unfolding always meant to come back - unless another line
            // also held, in which case there is still forward to go.
            None if self.queue.is_empty() => self.hand_back(BECAUSE_HOLE),
            None => self.resume_or_end(),
        }
    }

    /// Where a line leads, as a spot
    fn destination(&self, here: &str, index: usize) -> Option<Spot> {
        let node = at(self.convo, here)?;
        match node.lines.get(index)?.continues() {
            Continues::Here(_) => Some(format!("{}/l{}", here, index)),
            Continues::At(id) => Some(id.to_string()),
            Continues::Hole => None,
        }
    }

    /// The expectations that lost, each with the weight it was given
    /// before anything was said
    fn losers(&self, taken: &[usize], evidence: &Evidence) -> Vec<Rejected> {
        let node = self.node();
        node.lines
            .iter()
            .enumerate()
            .filter(|(i, _)| !taken.contains(i))
            .map(|(i, line)| Rejected {
                line: line.line.clone(),
                odds: crate::convo_route::authored_weight(node, i),
                mass: evidence.mass.get(&i).copied(),
            })
            .collect()
    }

    /// Flags a node sets on arrival, plus the terminal check
    fn arrive(&mut self) {
        let sets = self.node().set.clone();
        self.flags.extend(sets);
        if self.node().is_terminal() {
            self.resume_or_end();
        }
    }

    /// A node with no way out is the end, unless another line also held
    fn resume_or_end(&mut self) {
        if self.queue.is_empty() {
            if self.ending.is_none() {
                self.ending = Some(Ending::Done {
                    path: std::mem::take(&mut self.path),
                    fields: self.fields.clone(),
                    flags: self.flags.clone(),
                });
            }
            return;
        }
        let next = self.queue.remove(0);
        // Resuming cancels a hand-back caused by the first line running
        // out: the other lines are still forward.
        if let Some(Ending::BackToYou { path, .. }) = self.ending.take() {
            self.path = path;
        }
        self.path.push(Hop {
            at: self.spot.clone(),
            said: self.node().say.clone(),
            answered: None,
            took: None,
            odds: None,
            how: "queued",
            confidence: None,
            rejected: Vec::new(),
            also: Vec::new(),
            to: next.clone(),
            because: None,
        });
        self.spot = next;
        self.arrive();
    }
}

/// What the judgment put on each line, for the record
#[derive(Debug, Clone, Default)]
pub struct Evidence {
    pub confidence: Option<f64>,
    /// line index -> mass
    pub mass: HashMap<usize, f64>,
}

impl Evidence {
    /// Read the masses back out of an answer, keyed by line index
    pub fn from_answer(answer: &crate::route::RouteAnswer) -> Self {
        let mass = answer
            .probabilities
            .iter()
            .filter_map(|(k, v)| Some((crate::convo_route::index_of(k)?, *v)))
            .collect();
        Self {
            confidence: Some(answer.confidence),
            mass,
        }
    }
}

/// The result as the caller sees it
pub fn result(ending: &Ending) -> Value {
    json!(ending)
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
                {"line": "Quarantine it and move on", "goto": "quarantine",
                 "set": {"quarantined": true}},
                {"line": "Dig into why it's flaky",
                 "then": {"say": "Alright. Where do I start looking?",
                          "lines": [{"line": "The test itself"}, {"line": "The thing it's testing"}]}},
                {"line": "Delete it, it wasn't earning its keep", "odds": 1}]},
            "quarantine": {"say": "Fine. I'll mark it skipped with a note.",
              "ask": [{"text": "Issue number, if there is one", "id": "issue"}],
              "lines": [{"line": "Open an issue too"},
                        {"line": "No issue, just skip it", "when": "quarantined"}]}
          }}))
        .unwrap()
    }

    #[test]
    fn a_line_with_nothing_past_it_hands_back_rather_than_dead_ending() {
        let c = convo();
        let mut w = Walk::new(&c);
        w.pick(2); // "Delete it" - authored as one bare line
        let Some(Ending::BackToYou { because, path, .. }) = w.ending() else {
            panic!("expected a hand back");
        };
        assert_eq!(because, BECAUSE_HOLE);
        assert_eq!(path[0].to, HOLE);
        assert_eq!(
            path[0].took.as_deref(),
            Some("Delete it, it wasn't earning its keep")
        );
    }

    #[test]
    fn the_record_keeps_what_i_expected_as_well_as_what_happened() {
        let c = convo();
        let mut w = Walk::new(&c);
        let mut evidence = Evidence {
            confidence: Some(0.9),
            ..Default::default()
        };
        evidence.mass.insert(2, 0.05);
        w.apply(
            Move::Take {
                line: 2,
                why: Why::Matched,
            },
            Some("honestly just bin it".into()),
            &evidence,
        );
        let Some(Ending::BackToYou { path, .. }) = w.ending() else {
            panic!()
        };
        let hop = &path[0];
        // what they said, in their words, not mine
        assert_eq!(hop.answered.as_deref(), Some("honestly just bin it"));
        // and what I had thought of it beforehand: the lowest weight
        assert_eq!(hop.odds, Some(1));
        // the expectations that lost, with the weights I gave them
        let rejected: Vec<&str> = hop.rejected.iter().map(|r| r.line.as_str()).collect();
        assert_eq!(
            rejected,
            vec!["Quarantine it and move on", "Dig into why it's flaky"]
        );
        assert_eq!(hop.rejected[0].odds, 3);
    }

    #[test]
    fn a_goto_is_forward_motion_and_carries_its_flags() {
        let c = convo();
        let mut w = Walk::new(&c);
        w.pick(0);
        assert_eq!(w.spot(), "quarantine");
        assert_eq!(w.flags().get("quarantined"), Some(&json!(true)));
        // the gated line is open now that the flag is set
        assert_eq!(w.node().open_lines(&w.gate_values()).len(), 2);
    }

    #[test]
    fn an_inline_subtree_is_named_by_the_line_that_reached_it() {
        let c = convo();
        let mut w = Walk::new(&c);
        w.pick(1);
        assert_eq!(w.spot(), "open/l1");
        assert_eq!(w.node().say, "Alright. Where do I start looking?");
    }

    #[test]
    fn several_lines_at_once_are_queued_not_dropped() {
        let c = convo();
        let mut w = Walk::new(&c);
        w.apply(
            Move::Several {
                first: 1,
                rest: vec![0],
            },
            Some("both really".into()),
            &Evidence::default(),
        );
        assert_eq!(w.spot(), "open/l1");
        assert_eq!(w.path[0].also, vec!["Quarantine it and move on"]);
        // finishing the first branch resumes into the queued one
        w.pick(0);
        assert_eq!(w.spot(), "quarantine");
        assert!(w.ending().is_none());
    }

    #[test]
    fn undo_clears_the_node_instead_of_routing_on_a_misclick() {
        let c = convo();
        let mut w = Walk::new(&c);
        w.pick(0);
        w.fill("issue", json!("1234"));
        w.apply(
            Move::Undo,
            Some("ignore that, wrong number".into()),
            &Evidence::default(),
        );
        assert!(w.fields().get("issue").is_none());
        assert_eq!(w.spot(), "quarantine");
        assert!(w.ending().is_none());
    }

    #[test]
    fn a_terminal_node_finishes() {
        let c = convo();
        let mut w = Walk::new(&c);
        w.pick(0);
        w.pick(0); // "Open an issue too" - nothing past it, but it is a hole
        assert!(matches!(w.ending(), Some(Ending::BackToYou { .. })));
    }
}
