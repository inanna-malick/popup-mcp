use anyhow::Result;
use eframe::egui;
use egui::{
    CentralPanel, Color32, Context, Id, Key, Rect, RichText, ScrollArea, TopBottomPanel, Vec2,
};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use std::sync::{Arc, Mutex};

pub mod route_nav;

use crate::jev::JevClient;
use crate::theme::Theme;
use popup_common::route::{self, RouteAnswer, RouteDecision, EXIT_KEY};
use popup_common::SelectStyle;
use popup_common::{Edge, Element, PopupDefinition, PopupResult, PopupState};
use route_nav::{
    apply_prefill, build_path, is_graph, page_index, visited_active_ids, with_route_log,
};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Selects with at most this many options render as always-visible radio
/// lists so users can see every branch of the conversation tree.
/// 7 = 6 authored options plus the auto-injected "Other" (the documented
/// budget is "radio up to 6 options", counted pre-injection).
const RADIO_MAX_OPTIONS: usize = 7;

/// Content width: single readable column
const CONTENT_MAX_WIDTH: f32 = 560.0;

pub fn setup_custom_fonts(ctx: &Context) {
    // Install image loaders for egui-twemoji (required for emoji rendering)
    egui_extras::install_image_loaders(ctx);

    // Configure moderately larger text sizes
    let mut style = (*ctx.style()).clone();

    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(20.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(17.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(15.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::new(13.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::new(15.0, egui::FontFamily::Monospace),
    );

    ctx.set_style(style);
    log::info!("Installed image loaders for emoji support and configured text sizes");
}

pub fn render_popup(definition: PopupDefinition) -> Result<PopupResult> {
    let result = Arc::new(Mutex::new(None));
    let result_clone = result.clone();

    let title = definition.effective_title().to_string();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            // Start with a default size, it will be resized on the first frame
            .with_inner_size([CONTENT_MAX_WIDTH + 60.0, 320.0])
            // Allow the window to be resized by the user
            .with_resizable(true)
            .with_position(egui::Pos2::new(100.0, 100.0)),
        ..Default::default()
    };

    eframe::run_native(
        &title,
        options,
        Box::new(move |cc| {
            // Configure fonts for emoji support
            setup_custom_fonts(&cc.egui_ctx);

            let app = PopupApp::new_with_result(definition, result_clone);
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run eframe: {}", e))?;

    // Extract result
    let result = result
        .lock()
        .unwrap()
        .take()
        .ok_or_else(|| anyhow::anyhow!("Popup closed without result"))?;

    Ok(result)
}

/// A judgment in flight. The packet was built when the user pressed
/// Next, which is their commit: edits made while it runs are not seen.
struct RouteInFlight {
    /// Page the judgment is about, for the route log
    from: Option<String>,
    /// The page's semantic edges, in authored order
    edges: Vec<Edge>,
    rx: std::sync::mpsc::Receiver<Result<RouteAnswer, String>>,
}

/// Decision-tree navigation state (None for linear wizards and single
/// screens, which keep their existing behaviour)
struct GraphNav {
    /// Indices of the top-level pages visited, in order; last is current
    visited: Vec<usize>,
    in_flight: Option<RouteInFlight>,
    /// Pages still to open, from a judgment where several conditions held
    /// at once. Forward-only: the queue is just more forward.
    queue: Vec<String>,
    /// Why the current page opened, shown under its heading
    arrived: Option<String>,
    /// Element id -> the option inferred for it from what the user wrote
    inferred: HashMap<String, String>,
    /// One entry per hop, returned to the model as `_route`
    route_log: Vec<Value>,
    jev: Option<Arc<JevClient>>,
}

struct PopupApp {
    definition: PopupDefinition,
    state: PopupState,
    theme: Theme,
    result: Arc<Mutex<Option<PopupResult>>>,
    first_interactive_widget_id: Option<Id>,
    first_widget_focused: bool,
    last_size: Vec2,
    markdown_cache: CommonMarkCache,
    /// Index into the currently-visible pages (wizard mode)
    current_page: usize,
    /// Enter was pressed in a single-line input this frame
    advance_requested: bool,
    /// Wizard note text per top-level page index; composed into _notes as
    /// "[Page label] note" lines on submit
    page_notes: std::collections::HashMap<usize, String>,
    /// Decision-tree navigation, when any page declares `next` edges
    graph: Option<GraphNav>,
}

impl PopupApp {
    fn new_with_result(
        definition: PopupDefinition,
        result: Arc<Mutex<Option<PopupResult>>>,
    ) -> Self {
        let state = PopupState::new(&definition);
        // Graph mode: the popup is a decision tree, navigated by resolved
        // edges rather than by page order
        let graph = is_graph(&definition.elements).then(|| GraphNav {
            visited: vec![0],
            in_flight: None,
            queue: Vec::new(),
            arrived: None,
            inferred: HashMap::new(),
            route_log: Vec::new(),
            jev: JevClient::from_env().map(Arc::new),
        });
        Self {
            definition,
            state,
            theme: Theme::default(), // Uses solarized_dark
            result,
            first_interactive_widget_id: None,
            first_widget_focused: false,
            last_size: Vec2::ZERO, // Initialize to zero to force resize on first frame
            markdown_cache: CommonMarkCache::default(),
            current_page: 0,
            advance_requested: false,
            page_notes: std::collections::HashMap::new(),
            graph,
        }
    }

    /// Indices of currently-visible Page elements (empty = not a wizard)
    fn visible_pages(&self) -> Vec<usize> {
        let values = popup_common::visible_value_map(&self.definition.elements, &self.state);
        self.definition
            .elements
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                matches!(e, Element::Page { .. })
                    && popup_common::when_is_visible(e.when(), &values)
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// Fold per-page wizard notes into the single _notes string
    fn compose_page_notes(&mut self) {
        let lines: Vec<String> = self
            .definition
            .elements
            .iter()
            .enumerate()
            .filter_map(|(idx, element)| {
                let note = self.page_notes.get(&idx)?.trim();
                if note.is_empty() {
                    None
                } else {
                    Some(format!("[{}] {}", element.label(), note))
                }
            })
            .collect();
        if !lines.is_empty() {
            self.state.notes = lines.join("\n");
        }
    }

    /// Index of the page currently shown in graph mode
    fn graph_page(&self) -> usize {
        self.graph
            .as_ref()
            .and_then(|g| g.visited.last().copied())
            .unwrap_or(0)
    }

    /// Resolve the way out of the current page. Mechanical edges are
    /// taken in code; semantic ones start one judgment off the UI thread.
    /// The packet is built here, at the press, so a slow endpoint can
    /// never freeze the window and later edits cannot change the question
    /// that was asked.
    fn resolve_route(&mut self, ctx: &Context) {
        let idx = self.graph_page();
        let Some(page) = self.definition.elements.get(idx).cloned() else {
            self.finish(ctx);
            return;
        };
        let values = popup_common::visible_value_map(&self.definition.elements, &self.state);
        let visited = self
            .graph
            .as_ref()
            .map(|g| g.visited.clone())
            .unwrap_or_default();
        let path = build_path(&self.definition, &self.state, &visited, &self.page_notes);

        match route::decide(
            &self.definition.title,
            &self.definition.elements,
            &page,
            &values,
            &path,
        ) {
            RouteDecision::Submit => self.leave_page(ctx),
            RouteDecision::Mechanical { to } => {
                self.log_hop(json!({"from": page.page_id(), "to": to, "via": "when"}));
                self.navigate(&to, ctx);
            }
            RouteDecision::Ask(packet) => {
                let Element::Page { next, .. } = &page else {
                    return;
                };
                let edges = next.clone();
                let Some(client) = self.graph.as_ref().and_then(|g| g.jev.clone()) else {
                    // No key: there is nothing to judge with, so the
                    // conversation goes back to the model rather than
                    // asking the user to do the routing by hand
                    self.hand_back(
                        page.page_id(),
                        "error",
                        json!({"exit_reason": "unavailable",
                               "error": "no Jev key (TYPESAFE_API_KEY or ~/.config/typesafe/jev-key.env)"}),
                        ctx,
                    );
                    return;
                };
                let (tx, rx) = std::sync::mpsc::channel();
                let name = format!("route/{}", page.page_id().unwrap_or_default());
                let ctx_for_wake = ctx.clone();
                std::thread::spawn(move || {
                    let result = client
                        .ask(&packet, &name)
                        .and_then(|r| route::parse_route_answer(&r.body));
                    let _ = tx.send(result);
                    // Wake the event loop so the next page opens without
                    // waiting for an input event
                    ctx_for_wake.request_repaint();
                });
                if let Some(g) = self.graph.as_mut() {
                    g.in_flight = Some(RouteInFlight {
                        from: page.page_id(),
                        edges,
                        rx,
                    });
                }
            }
        }
    }

    /// Fold in a judgment that has landed and act on it at once. Returns
    /// true while still waiting. Forward-only: every branch either opens
    /// a page or ends the popup.
    fn poll_route(&mut self, ctx: &Context) -> bool {
        let Some(g) = self.graph.as_mut() else {
            return false;
        };
        let Some(in_flight) = g.in_flight.as_ref() else {
            return false;
        };
        let landed = match in_flight.rx.try_recv() {
            Err(std::sync::mpsc::TryRecvError::Empty) => return true,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                Err("the routing call did not return".to_string())
            }
            Ok(result) => result,
        };
        let in_flight = g.in_flight.take().expect("checked above");
        let from = in_flight.from.clone();

        let answer = match landed {
            Ok(a) => a,
            Err(e) => {
                log::warn!("routing unavailable: {}", e);
                self.hand_back(
                    from,
                    "error",
                    json!({"exit_reason": "unavailable", "error": e}),
                    ctx,
                );
                return false;
            }
        };

        let masses = json!(answer.probabilities);
        let holds = json!(answer.holds);
        // The note is the person's own words and outranks the widgets;
        // this says when it did, so the model can see which answer it
        // overrode rather than inferring it from the text
        let overridden = answer.contradicts >= 0.5;

        match answer.outcome(&in_flight.edges) {
            route::Outcome::Act {
                to,
                via,
                matched,
                confidence,
            } => {
                let prefilled = self.take_prefill(&to, &answer);
                self.log_hop(json!({
                    "from": from, "to": to, "via": via,
                    "matched": matched, "confidence": confidence,
                    "masses": masses, "holds": holds,
                    "prefilled": prefilled,
                    "note_overrides_answers": overridden,
                }));
                self.arrive(&matched, confidence);
                self.navigate(&to, ctx);
            }
            route::Outcome::Several { first, rest } => {
                let prefilled = self.take_prefill(&first, &answer);
                let matched = in_flight
                    .edges
                    .iter()
                    .find(|e| e.to == first)
                    .and_then(|e| e.r#if.clone())
                    .unwrap_or_default();
                self.log_hop(json!({
                    "from": from, "to": first, "via": "noul",
                    "matched": matched, "several": rest,
                    "masses": masses, "holds": holds,
                    "prefilled": prefilled,
                    "note_overrides_answers": overridden,
                }));
                self.arrive(&matched, answer.holds.get(&first).copied().unwrap_or(0.0));
                if let Some(g) = self.graph.as_mut() {
                    g.queue.extend(rest);
                }
                self.navigate(&first, ctx);
            }
            route::Outcome::Undo => {
                // They disowned what they filled in. Clearing the page and
                // letting them redo it beats routing on a mis-click, which
                // is what the edge-only packet did with this answer.
                let cleared = self.clear_current_page();
                self.log_hop(json!({
                    "from": from, "to": from, "via": "undo",
                    "cleared": cleared, "masses": masses, "holds": holds,
                }));
                if let Some(g) = self.graph.as_mut() {
                    g.arrived = Some("Cleared that - say it again however you like".to_string());
                    g.inferred.clear();
                }
                self.page_notes.remove(&self.graph_page());
            }
            route::Outcome::HandBack { reason, leaning } => {
                self.hand_back(
                    from,
                    "jev",
                    json!({
                        "exit_reason": reason, "leaning": leaning,
                        "confidence": answer.confidence,
                        "masses": masses, "holds": holds,
                        "note_overrides_answers": overridden,
                    }),
                    ctx,
                );
            }
        }
        false
    }

    /// Write the answers the judgment already read out of what the user
    /// wrote into the destination page's widgets, and remember which, so
    /// they can be marked as inferred rather than silently assumed
    fn take_prefill(&mut self, to: &str, answer: &RouteAnswer) -> Value {
        let Some(page) = page_index(&self.definition.elements, to)
            .and_then(|i| self.definition.elements.get(i))
            .cloned()
        else {
            return json!({});
        };
        let applied = apply_prefill(&self.definition, &mut self.state, &page, &answer.prefill);
        if let Some(g) = self.graph.as_mut() {
            g.inferred = applied.clone();
        }
        json!(applied)
    }

    /// Reset every answer on the current page to its default, and return
    /// the ids cleared so the result says what happened
    fn clear_current_page(&mut self) -> Vec<String> {
        let Some(page) = self.definition.elements.get(self.graph_page()).cloned() else {
            return Vec::new();
        };
        let fresh = PopupState::new(&self.definition);
        let mut cleared = Vec::new();
        for id in route_nav::ids_in_page(&page) {
            if let Some(value) = fresh.values.get(&id) {
                self.state.values.insert(id.clone(), value.clone());
                cleared.push(id);
            }
        }
        cleared
    }

    /// Record why the page about to open was chosen
    fn arrive(&mut self, matched: &str, confidence: f64) {
        if let Some(g) = self.graph.as_mut() {
            g.arrived = Some(format!(
                "Read your answers as \"{}\" ({:.0}%)",
                matched,
                confidence * 100.0
            ));
        }
    }

    fn log_hop(&mut self, entry: Value) {
        if let Some(g) = self.graph.as_mut() {
            if let Some(jev) = g.jev.as_ref() {
                jev.log_decision(&entry);
            }
            g.route_log.push(entry);
        }
    }

    /// End the popup and let the model continue the conversation. Every
    /// way of not having a branch lands here; `extra` says which.
    fn hand_back(&mut self, from: Option<String>, via: &str, extra: Value, ctx: &Context) {
        let mut entry = json!({"from": from, "to": EXIT_KEY, "via": via});
        if let (Some(entry), Some(extra)) = (entry.as_object_mut(), extra.as_object()) {
            for (k, v) in extra {
                entry.insert(k.clone(), v.clone());
            }
        }
        self.log_hop(entry);
        self.finish(ctx);
    }

    /// The current page has no way out: take the next queued page if a
    /// judgment found several branches, otherwise finish
    fn leave_page(&mut self, ctx: &Context) {
        let from = self
            .definition
            .elements
            .get(self.graph_page())
            .and_then(|p| p.page_id());
        let queued = self
            .graph
            .as_mut()
            .and_then(|g| (!g.queue.is_empty()).then(|| g.queue.remove(0)));
        match queued {
            Some(to) => {
                self.log_hop(json!({"from": from, "to": to, "via": "queue"}));
                if let Some(g) = self.graph.as_mut() {
                    g.arrived = Some("Also matched what you said".to_string());
                    g.inferred.clear();
                }
                self.navigate(&to, ctx);
            }
            None => self.finish(ctx),
        }
    }

    /// Move to a page by routing id, or finish
    fn navigate(&mut self, to: &str, ctx: &Context) {
        if to == popup_common::SUBMIT_TARGET {
            // An edge to "submit" ends this branch, but a queued branch
            // is still forward
            self.leave_page(ctx);
            return;
        }
        match page_index(&self.definition.elements, to) {
            Some(idx) => {
                if let Some(g) = self.graph.as_mut() {
                    g.visited.push(idx);
                    g.in_flight = None;
                }
            }
            None => {
                // Validation rejects unknown targets before rendering, so
                // this only fires on a bug; finishing beats a dead end
                log::error!("edge target \"{}\" has no page - finishing", to);
                self.finish(ctx);
            }
        }
    }

    fn finish(&mut self, ctx: &Context) {
        self.state.button_clicked = Some("submit".to_string());
        self.send_result_and_close(ctx);
    }

    fn send_result_and_close(&mut self, ctx: &Context) {
        let popup_result = if self.state.button_clicked.as_deref() == Some("cancel") {
            PopupResult::Cancelled
        } else {
            self.compose_page_notes();
            match &self.graph {
                // Graph mode: only pages actually walked contributed
                // answers, and the path taken rides along as `_route`
                Some(g) => {
                    let active_ids = visited_active_ids(&self.definition, &self.state, &g.visited);
                    let result =
                        PopupResult::from_state(&self.state, &self.definition, &active_ids);
                    with_route_log(result, &g.route_log)
                }
                None => {
                    let active_ids =
                        popup_common::collect_active_ids(&self.definition.elements, &self.state);
                    PopupResult::from_state(&self.state, &self.definition, &active_ids)
                }
            }
        };
        *self.result.lock().unwrap() = Some(popup_result);
        // Use ViewportCommand::Close to close the window
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

impl eframe::App for PopupApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Apply theme
        self.theme.apply_to_egui(ctx);

        // Handle Escape key for cancel
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.state.button_clicked = Some("cancel".to_string());
        }

        // Check if we should close
        if self.state.button_clicked.is_some() {
            self.send_result_and_close(ctx);
            return;
        }

        // --- Phase 1: Render UI and Measure Size ---

        // Navigation state. Graph mode walks resolved edges; the linear
        // wizard walks visible pages in order; a single screen does neither.
        let graph_mode = self.graph.is_some();
        // A judgment that has landed acts now: the next page opens by
        // itself, so this is where most navigation happens
        let waiting_on_route = self.poll_route(ctx);
        if self.state.button_clicked.is_some() {
            self.send_result_and_close(ctx);
            return;
        }
        let graph_page = self.graph_page();
        if waiting_on_route {
            // Keep frames coming so the spinner turns while we wait
            ctx.request_repaint();
        }
        let graph_terminal = matches!(
            self.definition.elements.get(graph_page),
            Some(Element::Page { next, .. }) if next.is_empty()
        );

        let visible_pages = if graph_mode {
            Vec::new()
        } else {
            self.visible_pages()
        };
        let is_wizard = !visible_pages.is_empty();
        if is_wizard && self.current_page >= visible_pages.len() {
            self.current_page = visible_pages.len() - 1;
        }
        // In graph mode "last page" means the current page has no way out
        // and no branch is queued behind it
        let queue_empty = self.graph.as_ref().is_none_or(|g| g.queue.is_empty());
        let on_last_page = if graph_mode {
            graph_terminal && queue_empty
        } else {
            !is_wizard || self.current_page + 1 == visible_pages.len()
        };
        // The page whose elements and note field are shown this frame
        let shown_page = if graph_mode {
            Some(graph_page)
        } else if is_wizard {
            Some(visible_pages[self.current_page])
        } else {
            None
        };

        // Keyboard flow: Cmd/Ctrl+Enter submits from anywhere; plain Enter
        // advances when no widget has focus, or when a single-line input
        // requested it last frame (advance_requested)
        let cmd_enter = ctx.input(|i| i.key_pressed(Key::Enter) && i.modifiers.command);
        let plain_enter_unfocused = ctx
            .input(|i| i.key_pressed(Key::Enter) && i.modifiers.is_none())
            && ctx.memory(|m| m.focused().is_none());

        if cmd_enter {
            self.state.button_clicked = Some("submit".to_string());
            self.send_result_and_close(ctx);
            return;
        }
        if self.advance_requested || plain_enter_unfocused {
            self.advance_requested = false;
            if graph_mode {
                if waiting_on_route {
                    // The question was asked; Enter does not ask it twice
                } else if graph_terminal {
                    self.leave_page(ctx);
                } else {
                    self.resolve_route(ctx);
                }
                if self.state.button_clicked.is_some() {
                    self.send_result_and_close(ctx);
                    return;
                }
                ctx.request_repaint();
                return;
            }
            if on_last_page {
                self.state.button_clicked = Some("submit".to_string());
                self.send_result_and_close(ctx);
                return;
            } else {
                self.current_page += 1;
            }
        }

        // Render the bottom panel (note field + navigation/submit) and get its height
        let submit_label = self.definition.submit_label().to_string();
        let show_notes = self.definition.notes;
        // Navigation requested by the button row, applied after the panel
        // closes so the borrow of `self` inside the closure ends first
        let mut pending_nav: Option<&'static str> = None;
        let bottom_panel_response = TopBottomPanel::bottom("submit_panel").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.separator();

            if show_notes {
                // Wizards get one note per page (bound to the current page);
                // single-screen popups get the global note
                let (header, note_text): (&str, &mut String) = match shown_page {
                    Some(page_idx) => (
                        "Add a note for this page (optional)",
                        self.page_notes.entry(page_idx).or_default(),
                    ),
                    None => ("Add a note (optional)", &mut self.state.notes),
                };
                egui::CollapsingHeader::new(
                    RichText::new(header)
                        .color(self.theme.text_secondary)
                        .size(13.0),
                )
                .default_open(false)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(note_text)
                            .desired_rows(2)
                            .desired_width(f32::INFINITY)
                            .text_color(self.theme.base2)
                            .hint_text("Anything this page didn't ask about"),
                    );
                });
            }

            ui.add_space(6.0);
            let mut nav: Option<&'static str> = None;
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.horizontal(|ui| {
                    // Center the button row
                    let stepped = is_wizard || graph_mode;
                    let row_width = if stepped { 340.0 } else { 140.0 };
                    let pad = ((ui.available_width() - row_width) / 2.0).max(0.0);
                    ui.add_space(pad);

                    if graph_mode {
                        // A decision tree only goes forward. The way out
                        // of a branch that does not fit is to hand the
                        // conversation back, not to walk backwards.
                        let out = egui::Button::new(
                            RichText::new("None of these")
                                .size(15.0)
                                .color(self.theme.text_secondary),
                        )
                        .min_size(egui::Vec2::new(150.0, 40.0));
                        if ui.add(out).clicked() {
                            nav = Some("handback");
                        }
                    } else if is_wizard {
                        let back = egui::Button::new(RichText::new("< Back").size(15.0))
                            .min_size(egui::Vec2::new(90.0, 40.0));
                        if ui.add_enabled(self.current_page > 0, back).clicked() {
                            nav = Some("back");
                        }
                        ui.label(
                            RichText::new(format!(
                                "{} / {}",
                                self.current_page + 1,
                                visible_pages.len()
                            ))
                            .color(self.theme.text_secondary),
                        );
                    }

                    if waiting_on_route {
                        // The judgment is running: the next page opens by
                        // itself, so there is nothing to press
                        ui.spinner();
                        ui.label(
                            RichText::new("Reading your answers...")
                                .color(self.theme.text_secondary)
                                .size(14.0),
                        );
                    } else {
                        let (label, action): (&str, &'static str) = if on_last_page {
                            (submit_label.as_str(), "submit")
                        } else {
                            ("Next >", "next")
                        };
                        let button_text = RichText::new(label)
                            .size(18.0)
                            .strong()
                            .color(self.theme.base2);
                        let button = egui::Button::new(button_text)
                            .min_size(egui::Vec2::new(140.0, 40.0))
                            .fill(self.theme.neon_pink.linear_multiply(0.2));
                        if ui.add(button).clicked() {
                            nav = Some(action);
                        }
                    }
                });
            });
            if let Some(action) = nav {
                pending_nav = Some(action);
            }
            ui.add_space(8.0);
        });
        let bottom_panel_height = bottom_panel_response.response.rect.height();

        // Apply navigation requested by the button row
        match pending_nav {
            Some("back") => {
                self.current_page -= 1;
                ctx.request_repaint();
                return;
            }
            Some("handback") => {
                let from = self
                    .definition
                    .elements
                    .get(self.graph_page())
                    .and_then(|p| p.page_id());
                self.hand_back(from, "user", json!({"exit_reason": "none_fit"}), ctx);
                return;
            }
            Some("next") => {
                if graph_mode {
                    self.resolve_route(ctx);
                    if self.state.button_clicked.is_some() {
                        self.send_result_and_close(ctx);
                        return;
                    }
                } else {
                    self.current_page += 1;
                }
                ctx.request_repaint();
                return;
            }
            Some("submit") => {
                self.state.button_clicked = Some("submit".to_string());
                self.send_result_and_close(ctx);
                return;
            }
            _ => {}
        }

        // Render the main content and measure its size
        CentralPanel::default().show(ctx, |ui| {
            // Add outer margin manually using a frame
            egui::Frame::NONE
                .inner_margin(egui::Margin::same(10))
                .show(ui, |ui| {
                    // Improved spacing for better readability
                    ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
                    ui.spacing_mut().button_padding = Vec2::new(10.0, 6.0);
                    ui.spacing_mut().indent = 16.0;

                    // Solid, space-reserving scrollbar: egui's default
                    // floating style is invisible until hovered, which reads
                    // as "content cut off" when a revealed branch grows past
                    // the window
                    ui.spacing_mut().scroll = egui::style::ScrollStyle::solid();
                    ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            // Use a scope to measure the content rect
                            let content_response = ui.scope(|ui| {
                                ui.set_max_width(CONTENT_MAX_WIDTH);
                                let inferred = self
                                    .graph
                                    .as_ref()
                                    .map(|g| g.inferred.clone())
                                    .unwrap_or_default();

                                // Wizard and graph modes render one page
                                let (page_elements, path_prefix) = match shown_page {
                                    Some(page_idx) => match &self.definition.elements[page_idx] {
                                        Element::Page { page, elements, .. } => {
                                            ui.label(
                                                RichText::new(page)
                                                    .color(self.theme.electric_blue)
                                                    .strong()
                                                    .size(18.0),
                                            );
                                            // Why this page opened. The
                                            // routing is invisible unless
                                            // it says what it read.
                                            if let Some(why) =
                                                self.graph.as_ref().and_then(|g| g.arrived.as_ref())
                                            {
                                                ui.label(
                                                    RichText::new(why)
                                                        .color(self.theme.text_secondary)
                                                        .size(12.0),
                                                );
                                            }
                                            ui.add_space(6.0);
                                            (elements.as_slice(), format!("p{}", page_idx))
                                        }
                                        _ => (self.definition.elements.as_slice(), String::new()),
                                    },
                                    None => (self.definition.elements.as_slice(), String::new()),
                                };

                                let mut render_ctx = RenderContext {
                                    theme: &self.theme,
                                    first_widget_id: &mut self.first_interactive_widget_id,
                                    widget_focused: self.first_widget_focused,
                                    markdown_cache: &mut self.markdown_cache,
                                    advance_requested: &mut self.advance_requested,
                                    inferred: &inferred,
                                };
                                render_elements(
                                    ui,
                                    page_elements,
                                    &mut self.state,
                                    &self.definition.elements,
                                    &mut render_ctx,
                                    &path_prefix,
                                );
                            });
                            // Store the measured rect in temporary memory to access it after the panel is drawn
                            ctx.memory_mut(|mem| {
                                mem.data.insert_temp(
                                    "content_rect".into(),
                                    content_response.response.rect,
                                )
                            });
                        });
                });
        });

        // --- Phase 2: Calculate Desired Size and Resize ---

        // Retrieve the content rect from memory
        let content_rect = ctx
            .memory(|mem| mem.data.get_temp::<Rect>("content_rect".into()))
            .unwrap_or(Rect::ZERO);

        let desired_width = content_rect.width() + ctx.style().spacing.window_margin.sum().x + 30.0;
        let desired_height = content_rect.height()
            + bottom_panel_height
            + ctx.style().spacing.window_margin.sum().y
            + 25.0; // Buffer to prevent scrollbar appearing unnecessarily

        // Clamp size to reasonable limits: one readable column, never taller
        // than the monitor (overflow scrolls instead)
        let max_height = ctx
            .input(|i| i.viewport().monitor_size)
            .map(|m| (m.y - 120.0).max(300.0))
            .unwrap_or(850.0)
            .min(850.0);
        let new_size = Vec2::new(
            desired_width.clamp(380.0, CONTENT_MAX_WIDTH + 80.0),
            desired_height.clamp(180.0, max_height),
        );

        // Resize the window if the desired size has changed
        if (new_size - self.last_size).length_sq() > 0.01 {
            self.last_size = new_size;
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(new_size));
        }

        // --- Phase 3: Handle Focus ---

        // Request focus on first interactive widget if not already focused
        if !self.first_widget_focused {
            if let Some(widget_id) = self.first_interactive_widget_id {
                ctx.memory_mut(|mem| mem.request_focus(widget_id));
                self.first_widget_focused = true;
            }
        }
    }
}

struct RenderContext<'a> {
    theme: &'a Theme,
    first_widget_id: &'a mut Option<Id>,
    widget_focused: bool,
    markdown_cache: &'a mut CommonMarkCache,
    /// Set when a single-line input receives Enter: advance to the next
    /// page, or submit on the last one
    advance_requested: &'a mut bool,
    /// Element ids answered in advance from what the user wrote. Marked,
    /// never hidden: the widget opens ticked and one click changes it.
    inferred: &'a HashMap<String, String>,
}

/// Says an answer was read from what the person already wrote, so a
/// ticked box they did not tick is accounted for rather than surprising
fn render_inferred_tag(ui: &mut egui::Ui, ctx: &RenderContext, id: &str) {
    if let Some(option) = ctx.inferred.get(id) {
        ui.label(
            RichText::new(format!("filled in from what you wrote: {}", option))
                .color(ctx.theme.text_secondary)
                .size(12.0)
                .italics(),
        );
    }
}

impl RenderContext<'_> {
    /// Remember the first interactive widget for initial keyboard focus
    fn note_first_widget(&mut self, id: Id) {
        if self.first_widget_id.is_none() && !self.widget_focused {
            *self.first_widget_id = Some(id);
        }
    }
}

/// Render a list of elements as a single ordered column.
///
/// Consecutive simple checkboxes (no reveals) are wrapped horizontally to
/// save vertical space; everything else flows top-to-bottom in author order.
fn render_elements(
    ui: &mut egui::Ui,
    elements: &[Element],
    state: &mut PopupState,
    all_elements: &[Element],
    ctx: &mut RenderContext,
    path_prefix: &str,
) {
    // Visibility-scoped value map: when-clauses only see values of elements
    // that are themselves visible (fixed-point in popup-common)
    let state_values = popup_common::visible_value_map(all_elements, state);

    // 1. Identify visible elements and their original indices
    let visible_indices: Vec<usize> = elements
        .iter()
        .enumerate()
        .filter(|(_, element)| popup_common::when_is_visible(element.when(), &state_values))
        .map(|(idx, _)| idx)
        .collect();

    if visible_indices.is_empty() {
        return;
    }

    // 2. Group consecutive simple checkboxes among visible elements
    let is_simple_check =
        |idx: usize| matches!(&elements[idx], Element::Check { reveals, .. } if reveals.is_empty());
    let mut items: Vec<Vec<usize>> = Vec::new();
    for idx in visible_indices {
        if is_simple_check(idx) {
            if let Some(last) = items.last_mut() {
                if last.iter().all(|&i| is_simple_check(i)) {
                    last.push(idx);
                    continue;
                }
            }
        }
        items.push(vec![idx]);
    }

    // 3. Render in author order, single column
    ui.vertical(|ui| {
        for item_indices in items {
            let is_check_run = item_indices.len() > 1
                || matches!(&elements[item_indices[0]], Element::Check { reveals, .. } if reveals.is_empty());

            if is_check_run && item_indices.len() > 1 {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 24.0;
                    ui.spacing_mut().item_spacing.y = 8.0;
                    for idx in &item_indices {
                        let element_path = element_path(path_prefix, *idx);
                        render_single_element(
                            ui,
                            &elements[*idx],
                            state,
                            all_elements,
                            ctx,
                            &element_path,
                        );
                    }
                });
            } else {
                for idx in &item_indices {
                    let element_path = element_path(path_prefix, *idx);
                    render_single_element(
                        ui,
                        &elements[*idx],
                        state,
                        all_elements,
                        ctx,
                        &element_path,
                    );
                }
            }
            ui.add_space(4.0);
        }
    });
}

fn element_path(prefix: &str, idx: usize) -> String {
    if prefix.is_empty() {
        idx.to_string()
    } else {
        format!("{}.{}", prefix, idx)
    }
}

/// Render an option description as small secondary text
fn render_option_description(ui: &mut egui::Ui, ctx: &RenderContext, desc: &str) {
    ui.horizontal(|ui| {
        ui.add_space(22.0); // Align under the radio/checkbox label
        ui.label(
            RichText::new(desc)
                .color(ctx.theme.text_secondary)
                .size(13.0),
        );
    });
}

fn render_single_element(
    ui: &mut egui::Ui,
    element: &Element,
    state: &mut PopupState,
    all_elements: &[Element],
    ctx: &mut RenderContext,
    element_path: &str,
) {
    // Visibility is decided by the caller (render_elements filters against
    // the visibility-scoped value map before dispatching here)
    match element {
        Element::Text { text, .. } => {
            // Use element path as unique ID to prevent collisions in conditionals
            ui.push_id(format!("text_{}", element_path), |ui| {
                ui.set_max_width(CONTENT_MAX_WIDTH);
                ui.label(RichText::new(text).color(ctx.theme.text_primary));
            });
        }

        Element::Markdown { markdown, .. } => {
            // Use element path as unique ID to prevent collisions in conditionals
            ui.push_id(format!("markdown_{}", element_path), |ui| {
                ui.set_max_width(CONTENT_MAX_WIDTH);
                CommonMarkViewer::new().show(ui, ctx.markdown_cache, markdown);
            });
        }

        Element::Multi {
            multi,
            id,
            options,
            option_children,
            reveals,
            ..
        } => {
            ui.vertical(|ui| {
                let has_descriptions = options.iter().any(|o| o.description().is_some());

                let selections_snapshot = if let Some(selections) = state.get_multichoice_mut(id) {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(multi)
                                .color(ctx.theme.matrix_green)
                                .strong()
                                .size(15.0),
                        );
                        if options.len() > 3 {
                            if ui.small_button("All").clicked() {
                                selections.iter_mut().for_each(|s| *s = true);
                            }
                            if ui.small_button("None").clicked() {
                                selections.iter_mut().for_each(|s| *s = false);
                            }
                        }
                    });
                    render_inferred_tag(ui, ctx, id);

                    ui.add_space(2.0);

                    let render_option =
                        |ui: &mut egui::Ui,
                         i: usize,
                         option: &popup_common::OptionValue,
                         selections: &mut Vec<bool>,
                         ctx: &mut RenderContext| {
                            ui.push_id(format!("multi_{}_{}", id, i), |ui| {
                                let mut value = selections[i];
                                let response = ui.checkbox(
                                    &mut value,
                                    RichText::new(option.value()).color(ctx.theme.matrix_green),
                                );
                                selections[i] = value;
                                if i == 0 {
                                    ctx.note_first_widget(response.id);
                                }
                            });
                        };

                    if has_descriptions {
                        // Single column with descriptions under each option
                        for (i, option) in options.iter().enumerate() {
                            if i >= selections.len() {
                                continue;
                            }
                            render_option(ui, i, option, selections, ctx);
                            if let Some(desc) = option.description() {
                                render_option_description(ui, ctx, desc);
                            }
                        }
                    } else {
                        // Compact two-column grid for plain options
                        egui::Grid::new(format!("multi_grid_{}", id))
                            .num_columns(2)
                            .spacing([24.0, 6.0])
                            .show(ui, |ui| {
                                for (i, option) in options.iter().enumerate() {
                                    if i < selections.len() {
                                        render_option(ui, i, option, selections, ctx);
                                    }
                                    if (i + 1) % 2 == 0 {
                                        ui.end_row();
                                    }
                                }
                            });
                    }

                    selections.clone()
                } else {
                    vec![]
                };

                for (i, option) in options.iter().enumerate() {
                    if i < selections_snapshot.len() && selections_snapshot[i] {
                        if let Some(children) = option_children.get(option.value()) {
                            ui.indent(format!("multiselect_cond_{}_{}", id, i), |ui| {
                                render_elements(
                                    ui,
                                    children,
                                    state,
                                    all_elements,
                                    ctx,
                                    &format!("{}.multiselect_{}", element_path, i),
                                );
                            });
                        }
                    }
                }

                let has_selection = selections_snapshot.iter().any(|&s| s);
                if has_selection && !reveals.is_empty() {
                    ui.indent(format!("multiselect_reveals_{}", id), |ui| {
                        render_elements(ui, reveals, state, all_elements, ctx, element_path);
                    });
                }
            });
        }

        Element::Select {
            select,
            id,
            options,
            style,
            option_children,
            reveals,
            ..
        } => {
            let use_radio = match style {
                Some(SelectStyle::Radio) => true,
                Some(SelectStyle::Dropdown) => false,
                None => options.len() <= RADIO_MAX_OPTIONS,
            };

            ui.vertical(|ui| {
                if use_radio {
                    ui.label(
                        RichText::new(select)
                            .color(ctx.theme.electric_blue)
                            .strong()
                            .size(15.0),
                    );
                    render_inferred_tag(ui, ctx, id);
                    for (idx, option) in options.iter().enumerate() {
                        // Avoid holding a mutable borrow across the loop:
                        // branch children below also need `state`
                        let is_selected = state.get_choice(id).flatten() == Some(idx);
                        ui.push_id(format!("radio_{}_{}", id, idx), |ui| {
                            let response = ui.radio(
                                is_selected,
                                RichText::new(option.value()).color(ctx.theme.base2),
                            );
                            if response.clicked() {
                                if let Some(selected) = state.get_choice_mut(id) {
                                    *selected = Some(idx);
                                }
                            }
                            if idx == 0 {
                                ctx.note_first_widget(response.id);
                            }
                        });
                        if let Some(desc) = option.description() {
                            render_option_description(ui, ctx, desc);
                        }
                        // Branch children render inline, directly under their
                        // option, so the follow-ups visually belong to the
                        // answer that revealed them
                        if state.get_choice(id).flatten() == Some(idx) {
                            if let Some(children) = option_children.get(option.value()) {
                                ui.indent(format!("choice_cond_{}_{}", id, idx), |ui| {
                                    render_elements(
                                        ui,
                                        children,
                                        state,
                                        all_elements,
                                        ctx,
                                        &format!("{}.choice_{}", element_path, idx),
                                    );
                                });
                            }
                        }
                    }
                } else {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(select)
                                .color(ctx.theme.electric_blue)
                                .strong()
                                .size(15.0),
                        );

                        if let Some(selected) = state.get_choice_mut(id) {
                            let selected_text = match *selected {
                                Some(idx) => {
                                    options.get(idx).map(|s| s.value()).unwrap_or("(invalid)")
                                }
                                None => "(none selected)",
                            };

                            let response = egui::ComboBox::from_id_salt(id)
                                .selected_text(RichText::new(selected_text).color(ctx.theme.base2))
                                .show_ui(ui, |ui| {
                                    if ui
                                        .selectable_label(selected.is_none(), "(none selected)")
                                        .clicked()
                                    {
                                        *selected = None;
                                    }
                                    for (idx, option) in options.iter().enumerate() {
                                        let response = ui.selectable_label(
                                            *selected == Some(idx),
                                            option.value(),
                                        );
                                        if let Some(desc) = option.description() {
                                            response.clone().on_hover_text(desc);
                                        }
                                        if response.clicked() {
                                            *selected = Some(idx);
                                        }
                                    }
                                });

                            ctx.note_first_widget(response.response.id);
                        }
                    });
                }

                let selected_option = state.get_choice(id).flatten();
                // Dropdown branch children render after the combo (the radio
                // path renders them inline under the selected option above)
                if !use_radio {
                    if let Some(idx) = selected_option {
                        if let Some(option_val) = options.get(idx) {
                            if let Some(children) = option_children.get(option_val.value()) {
                                ui.indent(format!("choice_cond_{}_{}", id, idx), |ui| {
                                    render_elements(
                                        ui,
                                        children,
                                        state,
                                        all_elements,
                                        ctx,
                                        &format!("{}.choice_{}", element_path, idx),
                                    );
                                });
                            }
                        }
                    }
                }

                if selected_option.is_some() && !reveals.is_empty() {
                    ui.indent(format!("choice_reveals_{}", id), |ui| {
                        render_elements(ui, reveals, state, all_elements, ctx, element_path);
                    });
                }
            });
        }

        Element::Check {
            check, id, reveals, ..
        } => {
            if let Some(value) = state.get_boolean_mut(id) {
                let check_text = RichText::new(check).color(ctx.theme.matrix_green).strong();
                let response = ui.checkbox(value, check_text);

                ctx.note_first_widget(response.id);

                if *value && !reveals.is_empty() {
                    ui.indent(format!("checkbox_reveals_{}", id), |ui| {
                        render_elements(
                            ui,
                            reveals,
                            state,
                            all_elements,
                            ctx,
                            &format!("{}.checkbox", element_path),
                        );
                    });
                }
            }
        }

        Element::Slider {
            slider,
            id,
            min,
            max,
            step,
            ..
        } => {
            ui.horizontal(|ui| {
                ui.set_min_height(24.0);
                let label_width = 140.0;
                ui.add_sized(
                    [label_width, 24.0],
                    egui::Label::new(
                        RichText::new(slider)
                            .color(ctx.theme.warning_orange)
                            .strong()
                            .size(15.0),
                    ),
                );

                if let Some(value) = state.get_number_mut(id) {
                    let available_width = ui.available_width();
                    let value_label_width = 70.0;
                    let slider_width = (available_width - value_label_width - 10.0).max(100.0);

                    let step_val = step.unwrap_or(1.0);
                    let whole_steps = step_val.fract() == 0.0 && min.fract() == 0.0;
                    let decimals = if whole_steps { 0 } else { 1 };

                    ui.spacing_mut().slider_width = slider_width;
                    let slider_widget = egui::Slider::new(value, *min..=*max)
                        .show_value(false)
                        .clamping(egui::SliderClamping::Always)
                        .step_by(step_val as f64)
                        .min_decimals(decimals)
                        .max_decimals(decimals.max(1));

                    let response = ui.add(slider_widget);

                    let value_text = if whole_steps {
                        format!("{:.0}/{:.0}", *value, *max)
                    } else {
                        format!("{:.1}/{:.1}", *value, *max)
                    };
                    ui.label(
                        RichText::new(value_text)
                            .color(ctx.theme.base2)
                            .text_style(egui::TextStyle::Small),
                    );

                    ctx.note_first_widget(response.id);
                }
            });
        }

        Element::Input {
            input,
            id,
            placeholder,
            rows,
            ..
        } => {
            // Subtle sunken background for inputs
            let widget_frame = egui::Frame::NONE
                .inner_margin(egui::Margin::symmetric(8, 4))
                .fill(ctx.theme.dark_gray);

            widget_frame.show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(input)
                            .color(ctx.theme.neon_purple)
                            .strong()
                            .size(15.0),
                    );

                    if let Some(value) = state.get_text_mut(id) {
                        let row_count = rows.unwrap_or(1);
                        let input_width = ui.available_width().min(CONTENT_MAX_WIDTH);

                        // Single-line inputs use a singleline widget so Enter
                        // advances the form instead of inserting a newline
                        let text_edit = if row_count <= 1 {
                            egui::TextEdit::singleline(value)
                                .text_color(ctx.theme.base2)
                                .desired_width(input_width)
                        } else {
                            egui::TextEdit::multiline(value)
                                .text_color(ctx.theme.base2)
                                .desired_width(input_width)
                                .min_size(Vec2::new(input_width, row_count as f32 * 24.0))
                        };

                        let response = if let Some(hint) = placeholder {
                            ui.add(text_edit.hint_text(hint))
                        } else {
                            ui.add(text_edit)
                        };
                        if row_count <= 1
                            && response.lost_focus()
                            && ui.input(|i| i.key_pressed(Key::Enter))
                        {
                            *ctx.advance_requested = true;
                        }
                        ctx.note_first_widget(response.id);
                    }
                });
            });
        }

        Element::Rank {
            rank, id, options, ..
        } => {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(rank)
                            .color(ctx.theme.neon_cyan)
                            .strong()
                            .size(15.0),
                    );
                    ui.label(
                        RichText::new("click in priority order")
                            .color(ctx.theme.text_secondary)
                            .size(12.0),
                    );
                });

                if let Some(order) = state.get_order_mut(id) {
                    for (i, option) in options.iter().enumerate() {
                        let position = order.iter().position(|&x| x == i);
                        let badge = match position {
                            Some(p) => format!("{}", p + 1),
                            None => "·".to_string(),
                        };
                        ui.push_id(format!("rank_{}_{}", id, i), |ui| {
                            let response = ui.selectable_label(
                                position.is_some(),
                                RichText::new(format!(" {}  {}", badge, option.value()))
                                    .color(ctx.theme.base2),
                            );
                            if response.clicked() {
                                match position {
                                    // Unrank: later items shift up
                                    Some(p) => {
                                        order.remove(p);
                                    }
                                    // Rank: next position
                                    None => order.push(i),
                                }
                            }
                            if i == 0 {
                                ctx.note_first_widget(response.id);
                            }
                        });
                        if let Some(desc) = option.description() {
                            render_option_description(ui, ctx, desc);
                        }
                    }

                    if !order.is_empty() {
                        let ranked = order.len();
                        ui.horizontal(|ui| {
                            if ui.small_button("Clear").clicked() {
                                order.clear();
                            }
                            if ranked < options.len() {
                                ui.label(
                                    RichText::new(format!(
                                        "{} of {} ranked - unranked are omitted",
                                        ranked,
                                        options.len()
                                    ))
                                    .color(ctx.theme.text_secondary)
                                    .size(12.0),
                                );
                            }
                        });
                    }
                }
            });
        }

        Element::Group {
            group, elements, ..
        } => {
            // Minimal ghost frame for groups
            let group_frame = egui::Frame::NONE
                .inner_margin(egui::Margin::same(8))
                .stroke(egui::Stroke::new(
                    1.0,
                    if ui.style().visuals.dark_mode {
                        Color32::from_rgb(88, 110, 117) // base01
                    } else {
                        Color32::from_rgb(147, 161, 161) // base1
                    },
                ));

            group_frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(group)
                            .color(ctx.theme.matrix_green)
                            .strong()
                            .size(16.0),
                    );
                });
                ui.add_space(4.0);
                render_elements(
                    ui,
                    elements,
                    state,
                    all_elements,
                    ctx,
                    &format!("{}.group", element_path),
                );
            });
        }

        Element::Page { .. } => {
            // Pages are handled by the wizard chrome in update(); validation
            // rejects nested pages, so this is unreachable in practice
            log::warn!("Page element reached the element renderer - skipping");
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn collect_active_elements_for_test(
        elements: &[Element],
        state: &PopupState,
        _all_elements: &[Element],
    ) -> Vec<String> {
        popup_common::collect_active_ids(elements, state)
    }
}
