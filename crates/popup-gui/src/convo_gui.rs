//! Rendering a conversation.
//!
//! A node is a turn: what the assistant said, then the ways it could go,
//! written as the person might say them. Picking one is the answer and
//! nothing is judged. Typing instead is what the judgment is for, and the
//! side panel shows what it made of the words as they land.
//!
//! The panel quotes the line verbatim, never a summary. The honesty of
//! the thing rests on the expectation having been written down before the
//! person spoke; a generated explanation would be narration that could
//! drift from the actual cause.

use anyhow::Result;
use eframe::egui::{self, Color32, Context, Key, RichText, ScrollArea, Vec2};
use popup_common::convo::{Conversation, Field};
use popup_common::convo_route::{authored_weight, decide, packet, Move, Standing};
use popup_common::convo_walk::{Evidence, Walk};
use popup_common::route::{self, RouteAnswer};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::jev::JevClient;
use crate::theme::Theme;

const WIDTH: f32 = 560.0;
const PANEL: f32 = 260.0;

/// Most times the panel will read before an answer is committed. The
/// judgment at commit is the one that matters and always happens;
/// everything before it is a courtesy, so it has a ceiling.
const MOST_READS: usize = 2;

/// Shortest fragment worth reading. A pause after two characters is
/// someone thinking mid-word, and a confident panel reading of "I" is
/// noise that also invites them to write towards whatever it guessed.
const ENOUGH: usize = 14;

/// Ignore a change that has not moved the text much: a comma added to a
/// sentence already read is not worth reading again.
const ENOUGH_CHANGE: usize = 8;

pub fn run(convo: Conversation) -> Result<Value> {
    let out = Arc::new(Mutex::new(None));
    let sink = out.clone();
    let title = convo.title.clone();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([WIDTH + PANEL, 420.0])
            .with_resizable(true),
        ..Default::default()
    };
    eframe::run_native(
        &title,
        options,
        Box::new(move |cc| {
            crate::gui::setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(ConvoApp::new(convo, sink)))
        }),
    )
    .map_err(|e| anyhow::anyhow!("failed to open the window: {}", e))?;

    let value = out.lock().unwrap().take();
    value.ok_or_else(|| anyhow::anyhow!("window closed without a result"))
}

/// A judgment running off the UI thread
struct InFlight {
    rx: std::sync::mpsc::Receiver<Result<RouteAnswer, String>>,
    /// The words it was asked about, so a stale answer can be dropped
    about: String,
}

/// What the panel is showing
#[derive(Default)]
struct Reading {
    /// The words this reading is about
    about: String,
    /// (line text, mass), most likely first
    lines: Vec<(String, f64)>,
    /// Index of the line it would take, when it clears the gate
    would_take: Option<usize>,
    /// What it would do instead, when it would not
    otherwise: Option<String>,
    error: Option<String>,
}

struct ConvoApp {
    convo: Conversation,
    walk: Walk<'static>,
    typed: String,
    in_flight: Option<InFlight>,
    reading: Reading,
    /// The words the last answer was about, and the answer itself. The
    /// panel and the transition are the same call: by the time the typing
    /// has settled the answer is usually already here, so committing is
    /// instant and costs nothing extra.
    last: Option<(String, RouteAnswer)>,
    /// Reads spent on the current node, against `MOST_READS`
    reads: usize,
    jev: Option<Arc<JevClient>>,
    theme: Theme,
    out: Arc<Mutex<Option<Value>>>,
    focused: bool,
}

impl ConvoApp {
    fn new(convo: Conversation, out: Arc<Mutex<Option<Value>>>) -> Self {
        // The walk borrows the conversation for the life of the window,
        // which outlives every frame; leaking once is cheaper and clearer
        // than threading a lifetime through the whole app.
        let leaked: &'static Conversation = Box::leak(Box::new(convo.clone()));
        Self {
            convo,
            walk: Walk::new(leaked),
            typed: String::new(),
            in_flight: None,
            reading: Reading::default(),
            last: None,
            reads: 0,
            jev: JevClient::from_env().map(Arc::new),
            theme: Theme::default(),
            out,
            focused: false,
        }
    }

    fn standing(&self) -> Standing {
        let node = self.walk.node();
        let mut fields = std::collections::HashMap::new();
        for field in &node.ask {
            if let Some(value) = self.walk.fields().get(field.id()) {
                fields.insert(field.label().to_string(), value.clone());
            }
        }
        Standing {
            typed: Some(self.typed.trim().to_string()).filter(|t| !t.is_empty()),
            fields,
            flags: self.walk.flags().clone(),
            history: self.walk.history().to_vec(),
        }
    }

    /// Start a judgment on what has been typed so far. The same call
    /// serves the panel and the transition: by the time they stop typing
    /// the answer has usually landed, so taking a line is instant.
    fn read(&mut self, ctx: &Context) {
        let words = self.typed.trim().to_string();
        if words.chars().count() < ENOUGH {
            self.reading = Reading::default();
            return;
        }
        // Already read something close enough to this
        if let Some((about, _)) = &self.last {
            if about.len().abs_diff(words.len()) < ENOUGH_CHANGE
                && words.starts_with(&about[..about.len().min(words.len())])
            {
                return;
            }
        }
        let Some(client) = self.jev.clone() else {
            self.reading = Reading {
                about: words,
                error: Some("no judgment configured, so typing hands back".into()),
                ..Default::default()
            };
            return;
        };
        let p = packet(self.walk_convo(), self.walk.node(), &self.standing());
        let (tx, rx) = std::sync::mpsc::channel();
        let name = format!("convo/{}", self.walk.spot());
        let wake = ctx.clone();
        std::thread::spawn(move || {
            let answer = client
                .ask(&p, &name)
                .and_then(|r| route::parse_route_answer(&r.body));
            let _ = tx.send(answer);
            wake.request_repaint();
        });
        self.reads += 1;
        self.in_flight = Some(InFlight { rx, about: words });
    }

    fn walk_convo(&self) -> &Conversation {
        &self.convo
    }

    /// Fold in an answer that has landed. Returns true while waiting.
    fn poll(&mut self) -> bool {
        let Some(flight) = &self.in_flight else {
            return false;
        };
        match flight.rx.try_recv() {
            Err(std::sync::mpsc::TryRecvError::Empty) => return true,
            Err(_) => {
                self.reading.error = Some("the judgment did not come back".into());
                self.in_flight = None;
            }
            Ok(Err(e)) => {
                self.reading = Reading {
                    about: flight.about.clone(),
                    error: Some(e),
                    ..Default::default()
                };
                self.in_flight = None;
            }
            Ok(Ok(answer)) => {
                let about = flight.about.clone();
                self.in_flight = None;
                self.last = Some((about.clone(), answer.clone()));
                self.show(answer, about);
            }
        }
        false
    }

    /// Turn an answer into what the panel says
    fn show(&mut self, answer: RouteAnswer, about: String) {
        let node = self.walk.node();
        let open = node.open_lines(&self.walk.gate_values());
        let mut lines: Vec<(String, f64)> = open
            .iter()
            .map(|(i, l)| {
                (
                    l.line.clone(),
                    answer.mass(&popup_common::convo_route::key(*i)),
                )
            })
            .collect();
        lines.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let (would_take, otherwise) = match decide(&answer, &open) {
            Move::Take { line, .. } => (open.iter().position(|(i, _)| *i == line), None),
            Move::Several { first, rest } => (
                open.iter().position(|(i, _)| *i == first),
                Some(format!("and {} more of these", rest.len())),
            ),
            Move::Undo => (None, Some("clear what you filled in here".into())),
            Move::Back { reason, .. } => (None, Some(back_wording(reason).into())),
        };
        self.reading = Reading {
            about,
            lines,
            would_take,
            otherwise,
            error: None,
        };
    }

    /// Commit what has been typed: act on the judgment, or hand back.
    fn commit(&mut self, ctx: &Context) {
        let words = self.typed.trim().to_string();
        if words.is_empty() {
            return;
        }
        // Reuse the answer the panel already has when it is about these
        // exact words. Asking again would be a second call for a question
        // already answered, and two calls about one decision is the thing
        // that measured worst.
        let answer = match self.last.take() {
            Some((about, answer)) if about == words => Some(answer),
            _ => {
                self.read(ctx);
                while self.poll() {
                    std::thread::sleep(Duration::from_millis(20));
                }
                self.last.take().map(|(_, a)| a)
            }
        };
        let Some(answer) = answer else {
            self.walk.hand_back("unavailable");
            self.finish();
            return;
        };
        let open = self.walk.node().open_lines(&self.walk.gate_values());
        let decision = decide(&answer, &open);
        let evidence = Evidence::from_answer(&answer);
        self.walk.apply(decision, Some(words), &evidence);
        self.typed.clear();
        self.reads = 0;
        self.reading = Reading::default();
        self.last = None;
        if self.walk.ending().is_some() {
            self.finish();
        }
    }

    fn pick(&mut self, index: usize) {
        self.walk.pick(index);
        self.typed.clear();
        self.reads = 0;
        self.reading = Reading::default();
        self.last = None;
        if self.walk.ending().is_some() {
            self.finish();
        }
    }

    fn finish(&mut self) {
        if let Some(ending) = self.walk.ending() {
            *self.out.lock().unwrap() = Some(json!(ending));
        }
    }
}

fn back_wording(reason: &str) -> &'static str {
    match reason {
        "none_fit" => "hand back: none of these fit",
        "unclear" => "hand back: not enough to go on yet",
        "between_two" => "hand back: between two of these",
        "adjacent" => "hand back: close to one of these but not quite",
        _ => "hand back",
    }
}

impl eframe::App for ConvoApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.theme.apply_to_egui(ctx);

        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.walk.cancel();
            *self.out.lock().unwrap() = Some(json!({"status": "cancelled"}));
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        if self.out.lock().unwrap().is_some() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        let waiting = self.poll();
        if waiting {
            ctx.request_repaint();
        }

        let mut clicked: Option<usize> = None;
        let mut commit = false;
        let mut read_now = false;

        egui::SidePanel::right("reading")
            .exact_width(PANEL)
            .show(ctx, |ui| self.render_panel(ui, waiting));

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 10.0);
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(&self.walk.node().say)
                            .color(self.theme.base2)
                            .size(16.0),
                    );
                    ui.add_space(6.0);

                    let fields: Vec<&Field> = self.walk.open_fields();
                    for field in fields {
                        self.render_field(ui, field);
                    }

                    ui.add_space(4.0);
                    let open = self.walk.node().open_lines(&self.walk.gate_values());
                    for (index, line) in &open {
                        let leading = self.reading.would_take.is_some()
                            && self
                                .reading
                                .lines
                                .first()
                                .map(|(text, _)| text == &line.line)
                                .unwrap_or(false);
                        if self.render_line(ui, &line.line, leading) {
                            clicked = Some(*index);
                        }
                    }

                    ui.add_space(10.0);
                    ui.label(
                        RichText::new("or say it your way")
                            .color(self.theme.text_secondary)
                            .size(12.0),
                    );
                    let edit = ui.add(
                        egui::TextEdit::multiline(&mut self.typed)
                            .desired_rows(2)
                            .desired_width(f32::INFINITY)
                            .hint_text("whatever you actually want to say"),
                    );

                    if !self.focused {
                        edit.request_focus();
                        self.focused = true;
                    }
                    if edit.lost_focus() {
                        if ui.input(|i| i.key_pressed(Key::Enter)) {
                            commit = true;
                        } else {
                            // Every call is tied to something the person
                            // did. No timer reads text nobody has finished
                            // writing, so the cost of leaving this window
                            // open and walking away is exactly zero.
                            read_now = true;
                        }
                    }
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        let ready = !self.typed.trim().is_empty();
                        if ui
                            .add_enabled(
                                ready,
                                egui::Button::new(RichText::new("Say that").size(15.0))
                                    .min_size(Vec2::new(120.0, 34.0)),
                            )
                            .clicked()
                        {
                            commit = true;
                        }
                        if ui
                            .button(RichText::new("None of these").color(self.theme.text_secondary))
                            .clicked()
                        {
                            self.walk.hand_back("none_fit");
                            self.finish();
                        }
                    });
                    ui.add_space(8.0);
                });
        });

        if let Some(index) = clicked {
            self.pick(index);
        } else if commit {
            self.commit(ctx);
        } else if read_now && self.reads < MOST_READS {
            self.read(ctx);
        }
    }
}

impl ConvoApp {
    fn render_line(&self, ui: &mut egui::Ui, text: &str, leading: bool) -> bool {
        let colour = if leading {
            self.theme.electric_blue
        } else {
            self.theme.base2
        };
        let button = egui::Button::new(RichText::new(text).size(15.0).color(colour))
            .min_size(Vec2::new(ui.available_width().min(WIDTH - 40.0), 0.0))
            .wrap();
        ui.add(button).clicked()
    }

    fn render_field(&mut self, ui: &mut egui::Ui, field: &Field) {
        match field {
            Field::Text {
                text,
                id,
                placeholder,
                rows,
                ..
            } => {
                ui.label(
                    RichText::new(text)
                        .color(self.theme.electric_blue)
                        .size(14.0),
                );
                let mut value = self
                    .walk
                    .fields()
                    .get(id)
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let widget = egui::TextEdit::multiline(&mut value)
                    .desired_rows(rows.unwrap_or(1))
                    .desired_width(f32::INFINITY)
                    .hint_text(placeholder.as_deref().unwrap_or(""));
                if ui.add(widget).changed() {
                    self.walk.fill(id, json!(value));
                }
            }
            Field::Number {
                number,
                id,
                min,
                max,
                step,
                default,
                ..
            } => {
                ui.label(
                    RichText::new(number)
                        .color(self.theme.electric_blue)
                        .size(14.0),
                );
                let mut value = self
                    .walk
                    .fields()
                    .get(id)
                    .and_then(|v| v.as_f64())
                    .unwrap_or_else(|| default.unwrap_or((min + max) / 2.0));
                let mut slider = egui::Slider::new(&mut value, *min..=*max);
                if let Some(step) = step {
                    slider = slider.step_by(*step);
                }
                if ui.add(slider).changed() {
                    self.walk.fill(id, json!(value));
                }
            }
        }
    }

    /// What the window makes of the words so far. Every line stays on
    /// screen with its weight, so the ways this could have gone are
    /// visible rather than hidden behind the one that won.
    fn render_panel(&self, ui: &mut egui::Ui, waiting: bool) {
        ui.add_space(8.0);
        ui.label(
            RichText::new("what I'm reading")
                .color(self.theme.electric_blue)
                .strong()
                .size(14.0),
        );
        ui.add_space(6.0);

        if waiting {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(
                    RichText::new("reading...")
                        .color(self.theme.text_secondary)
                        .size(12.0),
                );
            });
            return;
        }
        if let Some(error) = &self.reading.error {
            ui.label(
                RichText::new(error)
                    .color(self.theme.text_secondary)
                    .size(12.0),
            );
            return;
        }
        // A reading for words that have since been replaced is stale;
        // showing it would put a confident sentence next to text it was
        // never about.
        if self.reading.about != self.typed.trim() {
            ui.label(
                RichText::new("reading that...")
                    .color(self.theme.text_secondary)
                    .size(12.0),
            );
            return;
        }
        if self.reading.lines.is_empty() {
            ui.label(
                RichText::new(
                    "Pick one of the lines, or start typing and I'll say what I make of it.",
                )
                .color(self.theme.text_secondary)
                .size(12.0),
            );
            ui.add_space(10.0);
            self.render_expectations(ui);
            return;
        }

        if let Some(top) = self.reading.would_take {
            if let Some((text, mass)) = self.reading.lines.get(top) {
                ui.label(
                    RichText::new("you seem to be saying")
                        .color(self.theme.text_secondary)
                        .size(11.0),
                );
                // Verbatim, never a summary: the expectation was written
                // before they spoke, and quoting it is what makes the
                // explanation the actual cause rather than a story.
                ui.label(RichText::new(text).color(self.theme.base2).size(13.0));
                ui.label(
                    RichText::new(format!("{:.0}%", mass * 100.0))
                        .color(self.theme.electric_blue)
                        .size(12.0),
                );
            }
        } else if let Some(otherwise) = &self.reading.otherwise {
            ui.label(
                RichText::new(otherwise)
                    .color(self.theme.text_secondary)
                    .size(13.0),
            );
        }

        ui.add_space(10.0);
        ui.separator();
        ui.label(
            RichText::new("the others")
                .color(self.theme.text_secondary)
                .size(11.0),
        );
        for (i, (text, mass)) in self.reading.lines.iter().enumerate() {
            if Some(i) == self.reading.would_take {
                continue;
            }
            ui.label(
                RichText::new(format!("{:.0}%  {}", mass * 100.0, text))
                    .color(Color32::from_gray(120))
                    .size(11.0),
            );
        }
    }

    /// Before anything is typed, the panel still has something honest to
    /// show: what I thought was likely, written down in advance.
    fn render_expectations(&self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("what I expected")
                .color(self.theme.text_secondary)
                .size(11.0),
        );
        let node = self.walk.node();
        let open = node.open_lines(&self.walk.gate_values());
        let total: u32 = open.iter().map(|(i, _)| authored_weight(node, *i)).sum();
        for (index, line) in &open {
            let share = authored_weight(node, *index) as f64 / total.max(1) as f64;
            ui.label(
                RichText::new(format!("{:.0}%  {}", share * 100.0, line.line))
                    .color(Color32::from_gray(120))
                    .size(11.0),
            );
        }
    }
}
