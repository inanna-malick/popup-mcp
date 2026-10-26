//! Decision-tree routing: which edge leaves a page.
//!
//! A page's `next` edges are a pool the authoring model built. Mechanical
//! edges (`when`) are decided here in code. Semantic edges (`if`) become
//! one Jev Choice over the pool with a described exit; this module builds
//! that packet and reads the answer back, but never talks to the network,
//! so the whole routing logic is testable against canned responses.
//!
//! Routing is forward-only: a judgment that clears the gate opens the next
//! page by itself. The user's recourse is the opt-out on every page, and
//! anything the gate does not clear hands back to the calling model rather
//! than stopping to ask.

use crate::{Edge, Element, SUBMIT_TARGET};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Exit: the answers are clear, but no authored condition describes them
pub const EXIT_NONE_FIT: &str = "none_fit";
/// Exit: the answers do not say enough to tell which condition holds
pub const EXIT_UNCLEAR: &str = "unclear";

/// Value recorded in the result when routing hands back to the model
pub const EXIT_KEY: &str = "none_of_these";

/// Acceptance floors for preselecting a suggestion. Gate on confidence
/// first, then mass, then margin (the order the Jev playbook prescribes):
/// confidence is distribution shape, mass is how much the winner actually
/// carries, margin separates a win from a near tie.
pub const CONFIDENCE_FLOOR: f64 = 0.4;
pub const MASS_FLOOR: f64 = 0.4;
pub const MARGIN_FLOOR: f64 = 0.1;

/// How many separate things the person is asking for, as a Score rung.
/// At or above this, the answer covers more than one branch. Measured:
/// this separates cleanly (2.75 for "all of them" against 0.2-1.2 for
/// every single-branch answer) where the per-edge Nouls do not - an
/// "all of these" answer scores each edge *lower* than a clear single
/// pick does, so no Noul floor can tell the two apart.
pub const MANY_FLOOR: f64 = 1.5;

/// Noul floor for reading the set of branches once the multiplicity Score
/// has already said there is more than one. Lower than `HOLD_FLOOR`
/// because the question is which of them, not whether any.
pub const HOLD_FLOOR_MANY: f64 = 0.45;

/// A per-edge Noul at or above this holds on its own. Measured: on an
/// "all of these" answer three edges scored 0.93-0.95 while the Choice
/// sat on `unclear`, and the Nouls moved the Choice by at most 0.05.
pub const HOLD_FLOOR: f64 = 0.7;

/// A prefilled answer at or above this is written into the destination
/// page's widget, marked as inferred. Below it the widget opens blank:
/// date arithmetic, which Jev cannot do, lands here at ~0.49.
pub const PREFILL_FLOOR: f64 = 0.5;

/// Exit option on every prefill question
pub const NOT_IMPLIED: &str = "not_implied";

/// The multiplicity Score is only read when it is sure of itself. Measured
/// across four trees: without this floor, a Score of 1.6 at confidence 0.00
/// on a page where nothing was answered opened two branches. With it, the
/// same 51 cases went from 3 wrong to 1. The plateau runs 0.3 to 0.5.
pub const MANY_CONFIDENCE_FLOOR: f64 = 0.4;

/// Noul floor for the modal actions that are not page transitions
pub const ACTION_FLOOR: f64 = 0.7;
/// The narrowing action is read more readily: being between two branches
/// is a weaker claim than disowning an answer
pub const NARROW_FLOOR: f64 = 0.6;

/// How many earlier pages travel with the packet. Accuracy falls as the
/// state fills with detail the question does not need, and the decision
/// is about the current page's answers.
const EARLIER_PAGES_KEPT: usize = 3;

/// Destination questions previewed per branch. Measured neutral on
/// accuracy, so it stays small: three labels say where a branch leads
/// without padding the state.
const PREVIEW_QUESTIONS: usize = 3;

/// Destination widgets asked about per packet, across all edges
const PREFILL_QUESTIONS: usize = 8;

/// True for either exit key
pub fn is_exit(key: &str) -> bool {
    key == EXIT_NONE_FIT || key == EXIT_UNCLEAR || key == EXIT_KEY
}

/// One answered page on the path so far, as Jev sees it
#[derive(Debug, Clone, Serialize)]
pub struct PageAnswers {
    pub page: String,
    /// label -> value, using human-facing labels rather than ids so the
    /// edge descriptions (written in words) line up with the state
    pub answers: HashMap<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// A request body for the Jev endpoint, minus the model name
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RoutePacket {
    pub state: Value,
    pub questions: Value,
}

/// The parsed Choice answer for the `route` question
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RouteChoice {
    pub choice: String,
    pub probabilities: HashMap<String, f64>,
    pub confidence: f64,
}

/// Everything one packet came back with: the route Choice, the per-edge
/// Nouls, the note-contradiction Noul, and any destination answers the
/// person's own words already implied.
#[derive(Debug, Clone, PartialEq)]
pub struct RouteAnswer {
    pub choice: String,
    pub probabilities: HashMap<String, f64>,
    pub confidence: f64,
    /// edge key -> "this condition holds on its own"
    pub holds: HashMap<String, f64>,
    /// "the note corrects or contradicts a recorded answer"
    pub contradicts: f64,
    /// how many branches the person is asking for, as a Score rung
    pub how_many: f64,
    /// confidence in `how_many`; the rung is ignored below the floor
    pub how_many_confidence: f64,
    /// the modal actions, by name: undo, narrow, expand, confirm
    pub actions: HashMap<String, f64>,
    /// destination element id -> (option text, mass), already filtered to
    /// real options at or above `PREFILL_FLOOR`
    pub prefill: HashMap<String, (String, f64)>,
}

/// What to do with a judgment. Forward-only: either a page opens, or the
/// popup hands the conversation back to the model that opened it.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    /// Open this page. `via` says which signal decided it.
    Act {
        to: String,
        via: &'static str,
        matched: String,
        confidence: f64,
    },
    /// More than one condition holds on its own: open the first and queue
    /// the rest, in the order the author wrote them
    Several { first: String, rest: Vec<String> },
    /// Nothing safe to act on; finish and let the model read the answers
    HandBack {
        reason: &'static str,
        leaning: Option<String>,
    },
    /// The person disowned what they filled in: clear this page's answers
    /// and let them redo it, rather than routing on a mis-click
    Undo,
}

/// Hand-back reason: the Choice landed below the gate and no single edge
/// held on its own
pub const REASON_LOW_CONFIDENCE: &str = "low_confidence";
/// Hand-back reason: narrowed to two branches but not to one, so the
/// question that separates them is the assistant's to ask
pub const REASON_BETWEEN_TWO: &str = "between_two";
/// Hand-back reason: what they want sits next to a branch rather than on
/// one, so more choices would serve better than any page in the tree
pub const REASON_ADJACENT: &str = "adjacent";

/// What code decided from mechanical edges and, if needed, the Jev answer
#[derive(Debug, Clone, PartialEq)]
pub enum RouteDecision {
    /// A `when` edge held; no judgment needed
    Mechanical { to: String },
    /// Semantic edges exist; ask Jev with this packet
    Ask(RoutePacket),
    /// No edge applies and none is semantic: the page is terminal
    Submit,
}

/// Edge keys are the target ids; two edges to the same target would
/// collide, so the key carries the index when needed
pub fn edge_key(edges: &[Edge], idx: usize) -> String {
    let dup = edges.iter().filter(|e| e.to == edges[idx].to).count() > 1;
    if dup {
        format!("{}#{}", edges[idx].to, idx)
    } else {
        edges[idx].to.clone()
    }
}

/// Decide the way out of `page` given the visible value map for the
/// mechanical edges and the labelled path so far for the semantic ones.
pub fn decide(
    title: &str,
    elements: &[Element],
    page: &Element,
    visible_values: &HashMap<String, Value>,
    path: &[PageAnswers],
) -> RouteDecision {
    let Element::Page { next, .. } = page else {
        return RouteDecision::Submit;
    };
    for edge in next {
        if let Some(when) = &edge.when {
            if crate::when_is_visible(Some(when), visible_values) {
                return RouteDecision::Mechanical {
                    to: edge.to.clone(),
                };
            }
        }
    }
    if next.iter().any(|e| e.r#if.is_some()) {
        RouteDecision::Ask(route_packet(title, elements, page, path))
    } else {
        RouteDecision::Submit
    }
}

/// Build the Jev packet from the tree itself. The authoring model writes
/// one line per edge; everything else here is compiled from structure it
/// already declared, so no boilerplate reaches the tool schema.
///
/// Three things the compiled form does that a hand-written question would
/// have to repeat every time:
/// - each option is described by the *condition* that makes it hold, never
///   by the action it triggers (condition-described options measure far
///   better than action-described ones)
/// - each option carries what the destination page would go on to ask, so
///   the judgment can weigh where a branch leads instead of guessing from
///   its name (the taxonomy-walk pattern)
/// - the two ways of having no answer are separate options, because they
///   mean different things to the model that receives the result
pub fn route_packet(
    title: &str,
    elements: &[Element],
    page: &Element,
    path: &[PageAnswers],
) -> RoutePacket {
    let (label, edges) = match page {
        Element::Page { page, next, .. } => (page.as_str(), next.as_slice()),
        _ => ("", &[][..]),
    };
    let current = path.last();
    // Earlier pages carry answers only. Measured: full pages, options
    // only, and none at all gave identical choices and confidences, so
    // this keeps the cheapest form that still orients the judgment.
    let earlier: Vec<Value> = path
        .iter()
        .take(path.len().saturating_sub(1))
        .rev()
        .take(EARLIER_PAGES_KEPT)
        .map(|p| json!({"page": p.page, "answers": p.answers}))
        .collect();

    let mut criteria = serde_json::Map::new();
    for (i, edge) in edges.iter().enumerate() {
        let Some(cond) = &edge.r#if else { continue };
        let mut option = serde_json::Map::new();
        option.insert("take this branch when".to_string(), json!(cond));
        if edge.to == SUBMIT_TARGET {
            option.insert(
                "the branch then".to_string(),
                json!("ends the conversation here"),
            );
        } else if let Some(dest) = find_page(elements, &edge.to) {
            option.insert(
                "the branch then asks".to_string(),
                json!(question_labels(dest)),
            );
        }
        criteria.insert(
            format!("open:{}", edge_key(edges, i)),
            Value::Object(option),
        );
    }
    criteria.insert(
        EXIT_NONE_FIT.to_string(),
        json!({"take this branch when":
            "the answers and the note are clear, but what they describe is not any of the branches above"}),
    );
    criteria.insert(
        EXIT_UNCLEAR.to_string(),
        json!({"take this branch when":
            "the answers and the note do not say enough to tell which branch applies, or they support two of them equally"}),
    );

    // Everything a reference could point at, in the order it appeared on
    // screen. This is what makes "the second one you listed" and "that
    // last thing" resolve; measured as the single largest context gain
    // (dropping just the option list costs seven cases in fifty-one).
    let mut offered = Vec::new();
    for list in page.child_lists() {
        crate::walk_elements(list, &mut |e| {
            if let Some(options) = element_options(e) {
                for (i, opt) in options.iter().enumerate() {
                    offered.push(json!({
                        "position": i + 1,
                        "question": e.label(),
                        "option": match opt.description() {
                            Some(d) => format!("{} ({})", opt.value(), d),
                            None => opt.value().to_string(),
                        },
                    }));
                }
            }
        });
    }
    let mut said = Vec::new();
    for list in page.child_lists() {
        crate::walk_elements(list, &mut |e| {
            if let Element::Markdown { markdown, .. } = e {
                for line in markdown.lines() {
                    // Emphasis markers are typography, not content, and a
                    // literal reader should not have to see through them
                    let line = line.replace(['*', '#'], "");
                    let line = line.trim().trim_start_matches("- ").trim();
                    if !line.is_empty() {
                        said.push(json!({"position": said.len() + 1, "said": line}));
                    }
                }
            }
        });
    }
    let destinations: serde_json::Map<String, Value> = edges
        .iter()
        .filter(|e| e.r#if.is_some())
        .filter_map(|e| {
            let dest = find_page(elements, &e.to)?;
            Some((
                e.to.clone(),
                json!({"this branch then asks": question_labels(dest)}),
            ))
        })
        .collect();

    let mut state = serde_json::Map::from_iter([
        ("current_page".to_string(), json!(label)),
        (
            "current_answers".to_string(),
            current.map(|c| json!(c.answers)).unwrap_or(json!({})),
        ),
        (
            "current_note".to_string(),
            json!(current.and_then(|c| c.note.clone())),
        ),
        ("earlier_pages".to_string(), json!(earlier)),
    ]);
    if !offered.is_empty() {
        state.insert(
            "the choices this page offered, in order".to_string(),
            json!(offered),
        );
    }
    if !said.is_empty() {
        state.insert(
            "what the assistant said above the questions, in order".to_string(),
            json!(said),
        );
    }
    if !destinations.is_empty() {
        state.insert("where each branch goes".to_string(), json!(destinations));
    }
    if !title.is_empty() {
        state.insert("why this popup was opened".to_string(), json!(title));
    }
    let state = Value::Object(state);
    let mut questions = serde_json::Map::new();
    questions.insert("action".to_string(), json!({
            "type": "choice",
            "instructions": {
                "question": "What should this window do next?",
                "how to read": "Take the answers at face value. `current_note` is the person's own words and outranks a selection it disagrees with. A reference like \"the second one\" or \"that last thing\" points into the ordered lists in the state; match it by position or by wording.",
                "each option": "gives the condition for taking that branch and what that branch would go on to ask.",
                "earlier_pages": "background only; the decision is about the current page.",
            },
            "criteria": criteria,
    }));

    // Moves that are not page transitions. Kept out of the Choice pool on
    // purpose: the same actions crowded into one Choice measured worse
    // than a small pool with these riding alongside as Nouls.
    for (name, question) in [
        ("undo", "Is the person saying their filled-in answer was a mistake and should simply be cleared, rather than pointing at a different one instead?"),
        ("narrow", "Has the person narrowed it to two of the branches but not to one?"),
        ("expand", "Is the person asking for something close to one of the branches but not quite any of them, so more choices on this page would serve better than moving on?"),
        ("confirm", "Has the person already said enough to answer the questions on the branch that is about to open, so it can be shown pre-filled for a one-tap confirmation?"),
    ] {
        questions.insert(
            format!("is:{}", name),
            json!({"type": "noul", "instructions": question}),
        );
    }

    // One Noul per edge: "does this condition hold on its own". Two uses,
    // both measured. When the Choice fails the gate but exactly one edge
    // holds, that edge is a second path to action with evidence attached.
    // When several hold, the answer covers more than one branch, which a
    // single Choice can only report as `unclear`.
    for (i, edge) in edges.iter().enumerate() {
        let Some(cond) = &edge.r#if else { continue };
        questions.insert(
            format!("holds:{}", edge_key(edges, i)),
            json!({"type": "noul", "instructions":
                format!("Does this condition hold on its own for the current page: {}?", cond)}),
        );
    }

    // How many things they are asking for. Asked directly rather than
    // inferred from the per-edge Nouls, which cannot separate "all of
    // them" from one clear pick.
    questions.insert(
        "how_many".to_string(),
        json!({"type": "score",
               "instructions": "How many of the branches in `where each branch goes` is the person asking for? Several answers about the same branch still count as one.",
               "criteria": ["none of them", "one of them", "two of them", "three or more of them"]}),
    );

    // The note is the person's own words and outranks the widgets. This
    // says when it did, so the result can flag the answer it overrode.
    questions.insert(
        "contradicts".to_string(),
        json!({"type": "noul", "instructions":
            "Does `current_note` contradict or correct a recorded answer in `current_answers`?"}),
    );

    // Prefill: for the questions the likely destinations would ask, which
    // option the person has already implied. This removes typing, never a
    // question - a prefilled widget opens ticked and marked, and the user
    // changes it with one click.
    let mut asked = 0;
    for edge in edges.iter().filter(|e| e.r#if.is_some()) {
        if asked >= PREFILL_QUESTIONS {
            break;
        }
        let Some(dest) = find_page(elements, &edge.to) else {
            continue;
        };
        for el in dest.child_lists().into_iter().flatten() {
            if asked >= PREFILL_QUESTIONS {
                break;
            }
            let (Some(id), Some(options)) = (el.id(), prefill_options(el)) else {
                continue;
            };
            questions.insert(
                format!("prefill:{}", id),
                json!({
                    "type": "choice",
                    "instructions": format!(
                        "If `{}` were asked now, which option do the person's own words and earlier answers already imply?",
                        el.label()),
                    "criteria": options,
                }),
            );
            asked += 1;
        }
    }

    RoutePacket {
        state,
        questions: Value::Object(questions),
    }
}

/// The option pool of a select or multi, for the ordered context list
fn element_options(el: &Element) -> Option<&Vec<crate::OptionValue>> {
    match el {
        Element::Select { options, .. } | Element::Multi { options, .. } => Some(options),
        _ => None,
    }
}

/// The top-level page with this routing id
fn find_page<'a>(elements: &'a [Element], id: &str) -> Option<&'a Element> {
    elements.iter().find(|e| e.page_id().as_deref() == Some(id))
}

/// What a page asks, as the labels of its answerable elements. Static text
/// is left out: it frames the page but is not a question.
fn question_labels(page: &Element) -> Vec<String> {
    let mut labels = Vec::new();
    for list in page.child_lists() {
        crate::walk_elements(list, &mut |e| {
            if labels.len() < PREVIEW_QUESTIONS
                && !matches!(e, Element::Text { .. } | Element::Markdown { .. })
            {
                labels.push(e.label().to_string());
            }
        });
    }
    labels
}

/// The option pool for a prefillable widget: select and multi only, since
/// a number, a date, or free text is not something to infer (Jev cannot
/// do arithmetic, and a guessed sentence is worse than an empty box). The
/// auto-injected "Other" option is left out: it is an escape hatch, not a
/// thing the person's words can imply.
fn prefill_options(el: &Element) -> Option<serde_json::Map<String, Value>> {
    let options = match el {
        Element::Select { options, .. } | Element::Multi { options, .. } => options,
        _ => return None,
    };
    let mut pool = serde_json::Map::new();
    for opt in options {
        if opt
            .value()
            .eq_ignore_ascii_case(crate::validate::OTHER_LABEL)
            || opt.value().eq_ignore_ascii_case("other")
        {
            continue;
        }
        pool.insert(
            opt.value().to_string(),
            opt.description().map(|d| json!(d)).unwrap_or(Value::Null),
        );
    }
    if pool.is_empty() {
        return None;
    }
    pool.insert(
        NOT_IMPLIED.to_string(),
        json!("nothing said so far implies any option"),
    );
    Some(pool)
}

/// Read one packet's answers out of a raw Jev response body. Only the
/// route Choice is required; a response missing the companion questions
/// still routes, it just carries no extras.
pub fn parse_route_answer(body: &Value) -> Result<RouteAnswer, String> {
    let answers = body
        .get("answers")
        .ok_or_else(|| "response has no answers".to_string())?;
    let route = answers
        .get("action")
        .or_else(|| answers.get("route"))
        .ok_or_else(|| "response has no answers.action".to_string())?;
    let route: RouteChoice =
        serde_json::from_value(route.clone()).map_err(|e| format!("bad route answer: {}", e))?;

    let mut holds = HashMap::new();
    let mut contradicts = 0.0;
    let mut how_many = 0.0;
    let mut how_many_confidence = 0.0;
    let mut actions = HashMap::new();
    let mut prefill = HashMap::new();
    if let Some(map) = answers.as_object() {
        for (key, value) in map {
            if let Some(edge) = key.strip_prefix("holds:") {
                if let Some(n) = value.get("noul").and_then(Value::as_f64) {
                    holds.insert(edge.to_string(), n);
                }
            } else if key == "contradicts" {
                contradicts = value.get("noul").and_then(Value::as_f64).unwrap_or(0.0);
            } else if key == "how_many" {
                how_many = value.get("score").and_then(Value::as_f64).unwrap_or(0.0);
                how_many_confidence = value
                    .get("confidence")
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0);
            } else if let Some(name) = key.strip_prefix("is:") {
                if let Some(n) = value.get("noul").and_then(Value::as_f64) {
                    actions.insert(name.to_string(), n);
                }
            } else if let Some(id) = key.strip_prefix("prefill:") {
                let choice = value.get("choice").and_then(Value::as_str).unwrap_or("");
                let mass = value
                    .get("probabilities")
                    .and_then(|p| p.get(choice))
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0);
                // An implied answer is only worth writing into a widget
                // when it is one of the real options and carries weight
                if !choice.is_empty() && choice != NOT_IMPLIED && mass >= PREFILL_FLOOR {
                    prefill.insert(id.to_string(), (choice.to_string(), mass));
                }
            }
        }
    }

    Ok(RouteAnswer {
        choice: route.choice,
        probabilities: route.probabilities,
        confidence: route.confidence,
        holds,
        contradicts,
        how_many,
        how_many_confidence,
        actions,
        prefill,
    })
}

impl RouteAnswer {
    /// The edge key safe to act on: an edge (never an exit) that clears
    /// confidence, then mass, then margin. Confidence is the floor that
    /// does the work; the other two are cheap belts.
    pub fn suggestion(&self) -> Option<&str> {
        if is_exit(&self.choice)
            || self.confidence < CONFIDENCE_FLOOR
            || self.mass(&self.choice) < MASS_FLOOR
            || self.margin() < MARGIN_FLOOR
        {
            None
        } else {
            Some(self.choice.as_str())
        }
    }

    /// Key the wire uses for an edge in the action pool
    fn action_key(key: &str) -> String {
        format!("open:{}", key)
    }

    /// What the popup does with this judgment. Forward-only, so every
    /// branch either opens a page, clears the page, or ends the popup.
    ///
    /// The order is measured, not assumed (51 cases over four trees):
    /// disowning an answer comes first, because routing on an answer the
    /// person just took back is the worst outcome available. Multiplicity
    /// comes before the gate, because an answer covering several branches
    /// reads as `unclear` to a single Choice. The per-edge Nouls are the
    /// last resort, since they saturate on trees whose conditions share a
    /// shape.
    pub fn outcome(&self, edges: &[Edge]) -> Outcome {
        let semantic: Vec<(String, &Edge)> = edges
            .iter()
            .enumerate()
            .filter(|(_, e)| e.r#if.is_some())
            .map(|(i, e)| (edge_key(edges, i), e))
            .collect();
        let act = |name: &str| self.actions.get(name).copied().unwrap_or(0.0);

        if act("undo") >= ACTION_FLOOR {
            return Outcome::Undo;
        }

        // An answer covering several branches reads as `unclear` to one
        // Choice: not too little said, too much. Only trusted when the
        // Score is sure of itself; without that floor a rung of 1.6 at
        // confidence 0.00 opened two branches on an empty page.
        if self.how_many >= MANY_FLOOR && self.how_many_confidence >= MANY_CONFIDENCE_FLOOR {
            let held: Vec<&Edge> = semantic
                .iter()
                .filter(|(k, _)| self.hold(k) >= HOLD_FLOOR_MANY)
                .map(|(_, e)| *e)
                .collect();
            if let [first, rest @ ..] = held.as_slice() {
                if !rest.is_empty() {
                    return Outcome::Several {
                        first: first.to.clone(),
                        rest: rest.iter().map(|e| e.to.clone()).collect(),
                    };
                }
            }
        }

        if let Some(key) = self.suggestion() {
            if let Some((_, edge)) = semantic.iter().find(|(k, _)| Self::action_key(k) == key) {
                return Outcome::Act {
                    to: edge.to.clone(),
                    via: "jev",
                    matched: edge.r#if.clone().unwrap_or_default(),
                    confidence: self.confidence,
                };
            }
        }
        if is_exit(&self.choice) {
            return Outcome::HandBack {
                reason: self.exit_reason(),
                leaning: None,
            };
        }

        // Between two branches, or beside them: both are the assistant's
        // to resolve in chat, since the window cannot author a new question
        if act("narrow") >= NARROW_FLOOR {
            return Outcome::HandBack {
                reason: REASON_BETWEEN_TWO,
                leaning: self.leaning(&semantic),
            };
        }
        if act("expand") >= ACTION_FLOOR {
            return Outcome::HandBack {
                reason: REASON_ADJACENT,
                leaning: self.leaning(&semantic),
            };
        }

        let held: Vec<(&str, &Edge)> = semantic
            .iter()
            .filter(|(k, _)| self.hold(k) >= HOLD_FLOOR)
            .map(|(k, e)| (k.as_str(), *e))
            .collect();
        match held.as_slice() {
            [(key, only)] => Outcome::Act {
                to: only.to.clone(),
                via: "noul",
                matched: only.r#if.clone().unwrap_or_default(),
                confidence: self.hold(key),
            },
            [(_, first), rest @ ..] if !rest.is_empty() => Outcome::Several {
                first: first.to.clone(),
                rest: rest.iter().map(|(_, e)| e.to.clone()).collect(),
            },
            _ => Outcome::HandBack {
                reason: REASON_LOW_CONFIDENCE,
                leaning: self.leaning(&semantic),
            },
        }
    }

    /// "this condition holds on its own", for an edge key
    pub fn hold(&self, edge_key: &str) -> f64 {
        self.holds.get(edge_key).copied().unwrap_or(0.0)
    }

    /// Which branch the answer tilted towards, whatever the gate said
    fn leaning(&self, semantic: &[(String, &Edge)]) -> Option<String> {
        semantic
            .iter()
            .find(|(k, _)| Self::action_key(k) == self.choice)
            .map(|(_, e)| e.to.clone())
    }

    /// Mass on one key, for display
    pub fn mass(&self, key: &str) -> f64 {
        self.probabilities.get(key).copied().unwrap_or(0.0)
    }

    /// Combined mass of both exits: what the hand-back row is worth
    pub fn exit_mass(&self) -> f64 {
        self.mass(EXIT_NONE_FIT) + self.mass(EXIT_UNCLEAR) + self.mass(EXIT_KEY)
    }

    /// Which exit the answer leaned to, whether or not it won overall.
    /// The two mean different things downstream: no branch fit the answers
    /// versus the answers were too thin to tell.
    pub fn exit_reason(&self) -> &'static str {
        if self.mass(EXIT_UNCLEAR) > self.mass(EXIT_NONE_FIT) {
            EXIT_UNCLEAR
        } else {
            EXIT_NONE_FIT
        }
    }

    /// Distance between the leader and its nearest rival, where the two
    /// exits count as one rival. They are separate options because they
    /// mean different things to the caller, but both mean "do not open a
    /// page", so an edge has to beat them together.
    pub fn margin(&self) -> f64 {
        let leader = self
            .probabilities
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(k, v)| (k.clone(), *v));
        let Some((leader_key, top)) = leader else {
            return 0.0;
        };
        let rival = self
            .probabilities
            .iter()
            .filter(|(k, _)| **k != leader_key && !is_exit(k))
            .map(|(_, v)| *v)
            .fold(0.0, f64::max);
        let exits = if is_exit(&leader_key) {
            0.0
        } else {
            self.exit_mass()
        };
        top - rival.max(exits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PopupDefinition;

    fn tree() -> PopupDefinition {
        serde_json::from_str(
            r#"{"title": "T", "elements": [
              {"page": "Start", "elements": [{"input": "How's it going", "id": "vibe"}],
               "next": [
                 {"to": "fast", "if": "the user wants it done today"},
                 {"to": "slow", "if": "the user wants to think it over"},
                 {"to": "submit", "if": "the user wants nothing else"}]},
              {"page": "Fast path", "id": "fast", "elements": [
                 {"markdown": "**Heads up:** this skips review."},
                 {"check": "Skip the staging soak", "id": "skip_soak"},
                 {"slider": "Hours", "id": "hours", "min": 1, "max": 8}]},
              {"page": "Slow path", "id": "slow", "elements": [
                 {"select": "What's the hesitation", "id": "hesitation", "options": ["Risk", "Time"]}]}
            ]}"#,
        )
        .unwrap()
    }

    fn mech_tree() -> PopupDefinition {
        serde_json::from_str(
            r#"{"title": "T", "elements": [
              {"page": "Start", "elements": [{"check": "In a hurry", "id": "hurry"}],
               "next": [
                 {"to": "fast", "when": "hurry"},
                 {"to": "slow", "if": "the user is not in a hurry"}]},
              {"page": "Fast path", "id": "fast", "elements": []},
              {"page": "Slow path", "id": "slow", "elements": []}
            ]}"#,
        )
        .unwrap()
    }

    /// Edge keys travel prefixed in the action pool; exits do not
    fn wire(key: &str) -> String {
        if is_exit(key) {
            key.to_string()
        } else {
            format!("open:{}", key)
        }
    }

    fn answer(choice: &str, probs: &[(&str, f64)]) -> RouteAnswer {
        let probabilities: HashMap<String, f64> =
            probs.iter().map(|(k, v)| (wire(k), *v)).collect();
        let top = probabilities.values().copied().fold(0.0, f64::max);
        RouteAnswer {
            choice: wire(choice),
            probabilities,
            confidence: top,
            holds: HashMap::new(),
            contradicts: 0.0,
            how_many: 1.0,
            how_many_confidence: 0.9,
            actions: HashMap::new(),
            prefill: HashMap::new(),
        }
    }

    /// The semantic edges of the shared `tree()` fixture
    fn edges() -> Vec<Edge> {
        match &tree().elements[0] {
            Element::Page { next, .. } => next.clone(),
            _ => unreachable!(),
        }
    }

    fn with_holds(mut a: RouteAnswer, holds: &[(&str, f64)]) -> RouteAnswer {
        a.holds = holds.iter().map(|(k, v)| (k.to_string(), *v)).collect();
        a
    }

    #[test]
    fn mechanical_edge_wins_without_asking() {
        let def = mech_tree();
        let values = HashMap::from([("hurry".to_string(), json!(true))]);
        assert_eq!(
            decide(&def.title, &def.elements, &def.elements[0], &values, &[]),
            RouteDecision::Mechanical { to: "fast".into() }
        );
    }

    #[test]
    fn options_are_conditions_plus_where_the_branch_leads() {
        let def = tree();
        let path = vec![PageAnswers {
            page: "Start".into(),
            answers: HashMap::from([("How's it going".to_string(), json!("ship it"))]),
            note: None,
        }];
        let RouteDecision::Ask(packet) = decide(
            &def.title,
            &def.elements,
            &def.elements[0],
            &HashMap::new(),
            &path,
        ) else {
            panic!("expected Ask");
        };
        let criteria = &packet.questions["action"]["criteria"];

        // condition-described, never action-described
        assert_eq!(
            criteria["open:fast"]["take this branch when"],
            json!("the user wants it done today")
        );
        // the destination's questions travel with the option, so the
        // judgment can see where the branch goes; static text is excluded
        assert_eq!(
            criteria["open:fast"]["the branch then asks"],
            json!(["Skip the staging soak", "Hours"])
        );
        assert_eq!(
            criteria["open:submit"]["the branch then"],
            json!("ends the conversation here")
        );
        // both exits are present and described separately
        assert!(criteria[EXIT_NONE_FIT]["take this branch when"].is_string());
        assert!(criteria[EXIT_UNCLEAR]["take this branch when"].is_string());

        assert_eq!(
            packet.state["current_answers"]["How's it going"],
            json!("ship it")
        );
        assert_eq!(packet.state["earlier_pages"], json!([]));
    }

    #[test]
    fn earlier_pages_are_capped_to_recent_context() {
        let def = tree();
        let page = |n: usize| PageAnswers {
            page: format!("P{}", n),
            answers: HashMap::new(),
            note: None,
        };
        let path: Vec<PageAnswers> = (0..6).map(page).collect();
        let packet = route_packet(&def.title, &def.elements, &def.elements[0], &path);
        let earlier = packet.state["earlier_pages"].as_array().unwrap().clone();
        assert_eq!(earlier.len(), EARLIER_PAGES_KEPT);
        // most recent first, and never the current page
        assert_eq!(earlier[0]["page"], json!("P4"));
        assert_eq!(packet.state["current_answers"], json!({}));
    }

    #[test]
    fn no_edges_means_terminal() {
        let def = tree();
        assert_eq!(
            decide(
                &def.title,
                &def.elements,
                &def.elements[1],
                &HashMap::new(),
                &[]
            ),
            RouteDecision::Submit
        );
    }

    #[test]
    fn duplicate_targets_get_distinct_keys() {
        let edges = vec![
            Edge {
                to: "fast".into(),
                r#if: Some("a".into()),
                when: None,
            },
            Edge {
                to: "fast".into(),
                r#if: Some("b".into()),
                when: None,
            },
            Edge {
                to: "slow".into(),
                r#if: Some("c".into()),
                when: None,
            },
        ];
        assert_eq!(edge_key(&edges, 0), "fast#0");
        assert_eq!(edge_key(&edges, 1), "fast#1");
        assert_eq!(edge_key(&edges, 2), "slow");
    }

    #[test]
    fn answer_parsing_reads_the_route_question() {
        let body = json!({"answers": {"action": {
            "type": "choice", "choice": "open:fast",
            "probabilities": {"open:fast": 0.8, "open:slow": 0.15, EXIT_NONE_FIT: 0.03, EXIT_UNCLEAR: 0.02},
            "confidence": 0.77
        }}});
        let a = parse_route_answer(&body).unwrap();
        assert_eq!(a.suggestion(), Some("open:fast"));
        assert!((a.mass("open:slow") - 0.15).abs() < 1e-9);
        assert!((a.exit_mass() - 0.05).abs() < 1e-9);
        assert!(parse_route_answer(&json!({"answers": {}})).is_err());
    }

    #[test]
    fn suggestion_gates_on_confidence_then_mass_then_margin() {
        // a clear winner passes
        assert_eq!(
            answer("fast", &[("fast", 0.8), ("slow", 0.2)]).suggestion(),
            Some("open:fast")
        );
        // a near tie fails on margin even though one nominally leads
        let tie = RouteAnswer {
            confidence: 0.9,
            ..answer("fast", &[("fast", 0.46), ("slow", 0.44), ("unclear", 0.1)])
        };
        assert!(tie.margin() < MARGIN_FLOOR);
        assert_eq!(tie.suggestion(), None);
        // a plurality winner carrying too little mass fails
        let thin = RouteAnswer {
            confidence: 0.9,
            ..answer(
                "fast",
                &[("fast", 0.35), ("slow", 0.15), ("a", 0.15), ("b", 0.35)],
            )
        };
        assert_eq!(thin.suggestion(), None);
        // low confidence fails whatever the shape
        let unsure = RouteAnswer {
            confidence: 0.3,
            ..answer("fast", &[("fast", 0.9), ("slow", 0.1)])
        };
        assert_eq!(unsure.suggestion(), None);
    }

    #[test]
    fn packet_carries_edge_nouls_a_contradiction_check_and_prefill() {
        let def = tree();
        let packet = route_packet(&def.title, &def.elements, &def.elements[0], &[]);
        let q = packet.questions.as_object().unwrap();

        // one Noul per semantic edge, keyed by the edge, asking whether
        // that condition holds on its own
        assert_eq!(q["holds:fast"]["type"], json!("noul"));
        assert!(q["holds:fast"]["instructions"]
            .as_str()
            .unwrap()
            .contains("the user wants it done today"));
        assert_eq!(q["holds:slow"]["type"], json!("noul"));
        assert_eq!(q["contradicts"]["type"], json!("noul"));

        // prefill covers the destinations' selects, never their sliders,
        // checks or inputs, and is keyed by element id
        assert!(q.contains_key("prefill:hesitation"));
        assert!(!q.contains_key("prefill:hours"));
        assert!(!q.contains_key("prefill:skip_soak"));
        let criteria = &q["prefill:hesitation"]["criteria"];
        assert!(criteria.get("Risk").is_some());
        assert_eq!(
            criteria[NOT_IMPLIED],
            json!("nothing said so far implies any option")
        );
        // the question names itself, never the answer
        assert!(q["prefill:hesitation"]["instructions"]
            .as_str()
            .unwrap()
            .contains("What's the hesitation"));
    }

    #[test]
    fn parsing_reads_every_group_and_filters_weak_prefill() {
        let body = json!({"answers": {
            "action": {"choice": "open:fast", "probabilities": {"open:fast": 0.8, "open:slow": 0.2}, "confidence": 0.9},
            "holds:fast": {"noul": 0.93},
            "holds:slow": {"noul": 0.12},
            "contradicts": {"noul": 0.98},
            // implied strongly enough to write into the widget
            "prefill:hesitation": {"choice": "Risk", "probabilities": {"Risk": 0.87, NOT_IMPLIED: 0.13}},
            // nothing implied it: the widget opens blank
            "prefill:mood": {"choice": NOT_IMPLIED, "probabilities": {NOT_IMPLIED: 0.99}},
            // a real option, but under the floor (this is where date
            // arithmetic lands, and Jev cannot do arithmetic)
            "prefill:how_long": {"choice": "months", "probabilities": {"months": 0.49}},
        }});
        let a = parse_route_answer(&body).unwrap();
        assert_eq!(a.holds["fast"], 0.93);
        assert_eq!(a.contradicts, 0.98);
        assert_eq!(a.prefill["hesitation"].0, "Risk");
        assert!(!a.prefill.contains_key("mood"));
        assert!(!a.prefill.contains_key("how_long"));
    }

    #[test]
    fn a_response_missing_the_companions_still_routes() {
        let body = json!({"answers": {"action":
            {"choice": "open:fast", "probabilities": {"open:fast": 0.8, "open:slow": 0.2}, "confidence": 0.9}}});
        let a = parse_route_answer(&body).unwrap();
        assert!(a.holds.is_empty() && a.prefill.is_empty() && a.contradicts == 0.0);
        assert!(matches!(
            a.outcome(&edges()),
            Outcome::Act { via: "jev", .. }
        ));
    }

    #[test]
    fn a_clear_choice_opens_its_page() {
        let a = answer(
            "fast",
            &[("fast", 0.82), ("slow", 0.1), (EXIT_UNCLEAR, 0.08)],
        );
        assert_eq!(
            a.outcome(&edges()),
            Outcome::Act {
                to: "fast".into(),
                via: "jev",
                matched: "the user wants it done today".into(),
                confidence: 0.82,
            }
        );
    }

    #[test]
    fn an_edge_must_beat_both_exits_together() {
        // 0.46 leads each exit separately but not their union, and both
        // exits mean the same thing here: do not open a page
        let split = RouteAnswer {
            confidence: 0.9,
            ..answer(
                "fast",
                &[
                    ("fast", 0.46),
                    ("slow", 0.04),
                    (EXIT_NONE_FIT, 0.25),
                    (EXIT_UNCLEAR, 0.25),
                ],
            )
        };
        assert!(split.margin() < MARGIN_FLOOR);
        assert!(matches!(
            split.outcome(&edges()),
            Outcome::HandBack {
                reason: REASON_LOW_CONFIDENCE,
                ..
            }
        ));
    }

    #[test]
    fn one_strong_noul_acts_where_the_choice_stalled() {
        // the measured case: the Choice fell below the floor, but exactly
        // one condition holds on its own, so that edge acts with its own
        // evidence instead of stalling
        let stalled = RouteAnswer {
            confidence: 0.33,
            ..answer(
                "fast",
                &[("fast", 0.51), ("slow", 0.3), (EXIT_UNCLEAR, 0.19)],
            )
        };
        assert_eq!(stalled.suggestion(), None);
        assert_eq!(
            with_holds(stalled, &[("fast", 0.9), ("slow", 0.2)]).outcome(&edges()),
            Outcome::Act {
                to: "fast".into(),
                via: "noul",
                matched: "the user wants it done today".into(),
                confidence: 0.9,
            }
        );
    }

    #[test]
    fn all_of_these_queues_the_branches_in_authored_order() {
        // the case hit twice in real use: the answer covers more than one
        // branch, which a single Choice can only report as `unclear`
        let a = RouteAnswer {
            confidence: 0.54,
            ..answer(
                EXIT_UNCLEAR,
                &[("fast", 0.3), ("slow", 0.3), (EXIT_UNCLEAR, 0.4)],
            )
        };
        let a = with_holds(a, &[("fast", 0.94), ("slow", 0.93), ("submit", 0.02)]);
        // an answer covering several branches reads as `unclear` to the
        // Choice, so the multiplicity Score has to be read before the exit
        assert!(matches!(a.outcome(&edges()), Outcome::HandBack { .. }));
        assert_eq!(
            RouteAnswer {
                how_many: 2.75,
                ..a
            }
            .outcome(&edges()),
            Outcome::Several {
                first: "fast".into(),
                rest: vec!["slow".into()],
            }
        );

        let leaning = RouteAnswer {
            confidence: 0.4,
            ..answer(
                "fast",
                &[("fast", 0.34), ("slow", 0.33), (EXIT_UNCLEAR, 0.33)],
            )
        };
        assert_eq!(
            with_holds(leaning, &[("fast", 0.94), ("slow", 0.93)]).outcome(&edges()),
            Outcome::Several {
                first: "fast".into(),
                rest: vec!["slow".into()],
            }
        );
    }

    #[test]
    fn nothing_holding_hands_back_with_what_it_leaned_to() {
        let a = RouteAnswer {
            confidence: 0.3,
            ..answer(
                "slow",
                &[("slow", 0.4), ("fast", 0.35), (EXIT_UNCLEAR, 0.25)],
            )
        };
        assert_eq!(
            a.outcome(&edges()),
            Outcome::HandBack {
                reason: REASON_LOW_CONFIDENCE,
                leaning: Some("slow".into()),
            }
        );
    }

    #[test]
    fn earlier_pages_carry_answers_without_notes() {
        let def = tree();
        let path = vec![
            PageAnswers {
                page: "Start".into(),
                answers: HashMap::from([("How's it going".to_string(), json!("fine"))]),
                note: Some("a note that only mattered on that page".into()),
            },
            PageAnswers {
                page: "Second".into(),
                answers: HashMap::new(),
                note: None,
            },
        ];
        let packet = route_packet(&def.title, &def.elements, &def.elements[0], &path);
        let earlier = &packet.state["earlier_pages"][0];
        assert_eq!(earlier["answers"]["How's it going"], json!("fine"));
        assert!(earlier.get("note").is_none());
    }

    #[test]
    fn exits_never_become_suggestions_but_keep_their_reason() {
        let none_fit = answer(EXIT_NONE_FIT, &[(EXIT_NONE_FIT, 0.9), ("fast", 0.1)]);
        assert_eq!(none_fit.suggestion(), None);
        assert_eq!(none_fit.exit_reason(), EXIT_NONE_FIT);

        let unclear = answer(
            EXIT_UNCLEAR,
            &[(EXIT_UNCLEAR, 0.7), (EXIT_NONE_FIT, 0.2), ("fast", 0.1)],
        );
        assert_eq!(unclear.suggestion(), None);
        assert_eq!(unclear.exit_reason(), EXIT_UNCLEAR);
    }
}
