//! Minimal Jev (TypeSafe System One) client for decision-tree routing.
//!
//! One blocking POST per routing decision; retries on 429/503/529 with
//! capped backoff. Every call is appended to a local ledger so routing
//! decisions can be replayed and audited without re-asking.
//!
//! Key: `TYPESAFE_API_KEY`, else `~/.config/typesafe/jev-key.env`
//! (`KEY=value` lines or a bare key). The key is never logged.

use popup_common::RoutePacket;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::{Duration, Instant};

const ENDPOINT: &str = "https://api.typesafe.ai/v1/systemone";
const DEFAULT_MODEL: &str = "jev-latest";
const MAX_RETRIES: usize = 4;

#[derive(Debug)]
pub struct JevResponse {
    pub body: Value,
    pub latency_ms: u128,
}

pub struct JevClient {
    key: String,
    model: String,
    ledger: Option<PathBuf>,
}

impl JevClient {
    /// Build from the environment; `None` when no key is configured so the
    /// GUI can fall back to unaided user choice instead of failing
    pub fn from_env() -> Option<Self> {
        let key = std::env::var("TYPESAFE_API_KEY")
            .ok()
            .filter(|k| !k.trim().is_empty())
            .or_else(read_key_file)?;
        let model = std::env::var("JEV_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());
        let ledger = std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join(".config/popup-mcp/jev-ledger.jsonl"));
        Some(Self { key, model, ledger })
    }

    pub fn ask(&self, packet: &RoutePacket, name: &str) -> Result<JevResponse, String> {
        let body = json!({
            "state": packet.state,
            "model": self.model,
            "questions": packet.questions,
        });
        let mut delay = Duration::from_secs(1);
        let mut last_err = String::new();
        for _ in 0..MAX_RETRIES {
            let started = Instant::now();
            let resp = ureq::post(ENDPOINT)
                .set("Authorization", &format!("Bearer {}", self.key))
                .timeout(Duration::from_secs(30))
                .send_json(&body);
            let latency_ms = started.elapsed().as_millis();
            match resp {
                Ok(r) => {
                    let value: Value = r
                        .into_json()
                        .map_err(|e| format!("jev: malformed response body: {}", e))?;
                    self.log_row(json!({
                        "kind": "call",
                        "name": name,
                        "request": body,
                        "response": value,
                        "latency_ms": latency_ms,
                    }));
                    return Ok(JevResponse {
                        body: value,
                        latency_ms,
                    });
                }
                Err(ureq::Error::Status(code, r)) if matches!(code, 429 | 503 | 529) => {
                    last_err = format!("jev: HTTP {} ({})", code, r.status_text());
                    std::thread::sleep(delay);
                    delay = (delay * 2).min(Duration::from_secs(8));
                }
                Err(ureq::Error::Status(code, r)) => {
                    let text = r.into_string().unwrap_or_default();
                    return Err(format!(
                        "jev: HTTP {}: {}",
                        code,
                        text.chars().take(300).collect::<String>()
                    ));
                }
                Err(e) => return Err(format!("jev: transport error: {}", e)),
            }
        }
        Err(last_err)
    }

    /// What the gate did with the answer. Logged next to the call it came
    /// from, so the ledger alone says why a hop went where it did: without
    /// it a wrong branch is only diagnosable by re-deriving the decision.
    pub fn log_decision(&self, entry: &Value) {
        self.log_row(json!({"kind": "decision", "entry": entry}));
    }

    fn log_row(&self, mut row: Value) {
        let Some(path) = &self.ledger else { return };
        if let Some(obj) = row.as_object_mut() {
            obj.insert(
                "ts".to_string(),
                json!(std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs_f64())
                    .unwrap_or(0.0)),
            );
        }
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            use std::io::Write;
            let _ = writeln!(f, "{}", row);
        }
    }
}

fn read_key_file() -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let text =
        std::fs::read_to_string(PathBuf::from(home).join(".config/typesafe/jev-key.env")).ok()?;
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            l.rsplit_once('=')
                .map(|(_, v)| v)
                .unwrap_or(l)
                .trim()
                .trim_matches(|c| c == '"' || c == '\'')
                .to_string()
        })
        .find(|k| !k.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use popup_common::route::{self, PageAnswers, RouteDecision};
    use popup_common::PopupDefinition;
    use std::collections::HashMap;

    fn tree() -> PopupDefinition {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/incident_triage_tree.json"
        );
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    fn answers(page: &str, pairs: &[(&str, Value)], note: Option<&str>) -> PageAnswers {
        PageAnswers {
            page: page.to_string(),
            answers: pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
            note: note.map(str::to_string),
        }
    }

    /// Live routing check against the example tree. Ignored by default;
    /// run with `cargo test -p popup-gui -- --ignored` and a key present.
    #[test]
    #[ignore]
    fn live_routing_picks_the_described_edge() {
        let Some(client) = JevClient::from_env() else {
            panic!("no Jev key configured");
        };
        let def = tree();
        let page = &def.elements[0];

        let cases: Vec<(&str, PageAnswers, &str)> = vec![
            (
                "outright failure routes to the outage page",
                answers(
                    "What's happening",
                    &[
                        ("What are users seeing", json!("Checkout fails outright")),
                        ("A deploy went out in the last hour", json!(false)),
                    ],
                    None,
                ),
                "outage",
            ),
            (
                "slow but working routes to the degraded page",
                answers(
                    "What's happening",
                    &[
                        ("What are users seeing", json!("Checkout works but crawls")),
                        ("A deploy went out in the last hour", json!(false)),
                    ],
                    None,
                ),
                "degraded",
            ),
            (
                "a note describing total failure outweighs the milder selection",
                answers(
                    "What's happening",
                    &[
                        ("What are users seeing", json!("Checkout works but crawls")),
                        ("A deploy went out in the last hour", json!(false)),
                    ],
                    Some("correction: every payment is erroring now, nobody can check out"),
                ),
                "outage",
            ),
        ];

        for (name, page_answers, expected) in cases {
            let RouteDecision::Ask(packet) = route::decide(
                &def.title,
                &def.elements,
                page,
                &HashMap::new(),
                &[page_answers],
            ) else {
                panic!("{}: expected a semantic decision", name);
            };
            let response = client.ask(&packet, "test/live-routing").unwrap();
            let answer = route::parse_route_answer(&response.body).unwrap();
            assert_eq!(
                answer.suggestion(),
                Some(expected),
                "{}: got {:?} at confidence {:.2}",
                name,
                answer.choice,
                answer.confidence
            );
        }
    }
}
