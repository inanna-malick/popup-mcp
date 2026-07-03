use anyhow::Result;
use eframe::egui;
use egui::{
    CentralPanel, Color32, Context, Id, Key, Rect, RichText, ScrollArea, TopBottomPanel, Vec2,
};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use std::sync::{Arc, Mutex};

use crate::theme::Theme;
use popup_common::{evaluate_condition, parse_condition, SelectStyle};
use popup_common::{Element, PopupDefinition, PopupResult, PopupState};

/// Selects with at most this many options render as always-visible radio
/// lists so users can see every branch of the conversation tree.
/// 7 = 6 authored options plus the auto-injected "Other" (the documented
/// budget is "radio up to 6 options", counted pre-injection).
const RADIO_MAX_OPTIONS: usize = 7;

/// Content width: single readable column
const CONTENT_MAX_WIDTH: f32 = 560.0;

fn setup_custom_fonts(ctx: &Context) {
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

struct PopupApp {
    definition: PopupDefinition,
    state: PopupState,
    theme: Theme,
    result: Arc<Mutex<Option<PopupResult>>>,
    first_interactive_widget_id: Option<Id>,
    first_widget_focused: bool,
    last_size: Vec2,
    markdown_cache: CommonMarkCache,
}

impl PopupApp {
    fn new_with_result(
        definition: PopupDefinition,
        result: Arc<Mutex<Option<PopupResult>>>,
    ) -> Self {
        let state = PopupState::new(&definition);
        Self {
            definition,
            state,
            theme: Theme::default(), // Uses solarized_dark
            result,
            first_interactive_widget_id: None,
            first_widget_focused: false,
            last_size: Vec2::ZERO, // Initialize to zero to force resize on first frame
            markdown_cache: CommonMarkCache::default(),
        }
    }

    fn send_result_and_close(&mut self, ctx: &Context) {
        let popup_result = if self.state.button_clicked.as_deref() == Some("cancel") {
            PopupResult::Cancelled
        } else {
            // Collect only active element ids based on current state
            let active_ids = collect_active_elements(
                &self.definition.elements,
                &self.state,
                &self.definition.elements,
                "",
            );
            PopupResult::from_state(&self.state, &self.definition, &active_ids)
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

        // Render the bottom panel (note field + submit button) and get its height
        let submit_label = self.definition.submit_label().to_string();
        let show_notes = self.definition.notes;
        let bottom_panel_response = TopBottomPanel::bottom("submit_panel").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.separator();

            if show_notes {
                egui::CollapsingHeader::new(
                    RichText::new("Add a note (optional)")
                        .color(self.theme.text_secondary)
                        .size(13.0),
                )
                .default_open(false)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.state.notes)
                            .desired_rows(2)
                            .desired_width(f32::INFINITY)
                            .text_color(self.theme.base2)
                            .hint_text("Anything the form didn't ask about"),
                    );
                });
            }

            ui.add_space(6.0);
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                let button_text = RichText::new(&submit_label)
                    .size(18.0)
                    .strong()
                    .color(self.theme.base2);
                let button = egui::Button::new(button_text)
                    .min_size(egui::Vec2::new(140.0, 40.0))
                    .fill(self.theme.neon_pink.linear_multiply(0.2));

                if ui.add(button).clicked() {
                    self.state.button_clicked = Some("submit".to_string());
                }
            });
            ui.add_space(8.0);
        });
        let bottom_panel_height = bottom_panel_response.response.rect.height();

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
                                let mut render_ctx = RenderContext {
                                    theme: &self.theme,
                                    first_widget_id: &mut self.first_interactive_widget_id,
                                    widget_focused: self.first_widget_focused,
                                    markdown_cache: &mut self.markdown_cache,
                                };
                                render_elements(
                                    ui,
                                    &self.definition.elements,
                                    &mut self.state,
                                    &self.definition.elements,
                                    &mut render_ctx,
                                    "",
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
    let state_values = state.to_value_map(all_elements);

    // 1. Identify visible elements and their original indices
    let visible_indices: Vec<usize> = elements
        .iter()
        .enumerate()
        .filter(|(_, element)| match element.when() {
            None => true,
            Some(when_expr) => match parse_condition(when_expr) {
                Ok(ast) => evaluate_condition(&ast, &state_values),
                Err(_) => true, // validated upstream; fail-open here
            },
        })
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
    // Check if element should be visible based on when clause
    if let Some(when_expr) = element.when() {
        let state_values = state.to_value_map(all_elements);
        match parse_condition(when_expr) {
            Ok(ast) => {
                if !evaluate_condition(&ast, &state_values) {
                    // Condition not met - don't render this element
                    return;
                }
            }
            Err(e) => {
                // Definitions are validated before rendering; if something
                // slips through, render anyway (fail-open) but log it
                log::warn!("Failed to parse when clause '{}': {}", when_expr, e);
            }
        }
    }

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
                    if let Some(selected) = state.get_choice_mut(id) {
                        for (idx, option) in options.iter().enumerate() {
                            ui.push_id(format!("radio_{}_{}", id, idx), |ui| {
                                let response = ui.radio(
                                    *selected == Some(idx),
                                    RichText::new(option.value()).color(ctx.theme.base2),
                                );
                                if response.clicked() {
                                    *selected = Some(idx);
                                }
                                if idx == 0 {
                                    ctx.note_first_widget(response.id);
                                }
                            });
                            if let Some(desc) = option.description() {
                                render_option_description(ui, ctx, desc);
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
                        let height = rows.unwrap_or(1) as f32 * 24.0;
                        let input_width = ui.available_width().min(CONTENT_MAX_WIDTH);
                        let text_edit = egui::TextEdit::multiline(value)
                            .text_color(ctx.theme.base2)
                            .desired_width(input_width)
                            .min_size(Vec2::new(input_width, height));

                        let response = if let Some(hint) = placeholder {
                            ui.add(text_edit.hint_text(hint))
                        } else {
                            ui.add(text_edit)
                        };
                        ctx.note_first_widget(response.id);
                    }
                });
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
    }
}

// Helper functions

/// Collect only the active elements based on current state (evaluating when clauses)
fn collect_active_elements(
    elements: &[Element],
    state: &PopupState,
    all_elements: &[Element],
    _path_prefix: &str,
) -> Vec<String> {
    let mut active_ids = Vec::new();
    let state_values = state.to_value_map(all_elements);

    // Helper to check if an element's when clause is satisfied
    let is_visible = |when: Option<&str>| -> bool {
        match when {
            None => true, // No when clause means always visible
            Some(when_expr) => {
                // Parse and evaluate when clause
                match parse_condition(when_expr) {
                    Ok(ast) => evaluate_condition(&ast, &state_values),
                    Err(_) => {
                        // If parsing fails, default to visible (fail-open)
                        log::warn!("Failed to parse when clause: {}", when_expr);
                        true
                    }
                }
            }
        }
    };

    for element in elements {
        match element {
            Element::Slider { id, when, .. } | Element::Input { id, when, .. } => {
                if is_visible(when.as_deref()) {
                    active_ids.push(id.clone());
                }
            }
            Element::Check {
                id, reveals, when, ..
            } => {
                if is_visible(when.as_deref()) {
                    active_ids.push(id.clone());
                    // If checkbox is checked and has reveals, collect from it
                    if state.get_boolean(id) && !reveals.is_empty() {
                        active_ids.extend(collect_active_elements(
                            reveals,
                            state,
                            all_elements,
                            "",
                        ));
                    }
                }
            }
            Element::Multi {
                id,
                options,
                option_children,
                reveals,
                when,
                ..
            } => {
                if is_visible(when.as_deref()) {
                    active_ids.push(id.clone());
                    // For each checked option with children, collect from it
                    if let Some(selections) = state.get_multichoice(id) {
                        let has_selection = selections.iter().any(|&s| s);

                        for (i, option) in options.iter().enumerate() {
                            if i < selections.len() && selections[i] {
                                if let Some(children) = option_children.get(option.value()) {
                                    active_ids.extend(collect_active_elements(
                                        children,
                                        state,
                                        all_elements,
                                        "",
                                    ));
                                }
                            }
                        }

                        // Collect from reveals only if any option is selected
                        if has_selection && !reveals.is_empty() {
                            active_ids.extend(collect_active_elements(
                                reveals,
                                state,
                                all_elements,
                                "",
                            ));
                        }
                    }
                }
            }
            Element::Select {
                id,
                options,
                option_children,
                reveals,
                when,
                ..
            } => {
                if is_visible(when.as_deref()) {
                    active_ids.push(id.clone());

                    let has_selection = state
                        .get_choice(id)
                        .map(|opt| opt.is_some())
                        .unwrap_or(false);

                    // If there's a selected option with children, collect from it
                    if let Some(Some(idx)) = state.get_choice(id) {
                        if let Some(option_text) = options.get(idx) {
                            if let Some(children) = option_children.get(option_text.value()) {
                                active_ids.extend(collect_active_elements(
                                    children,
                                    state,
                                    all_elements,
                                    "",
                                ));
                            }
                        }
                    }

                    // Collect from reveals only if an option is selected
                    if has_selection && !reveals.is_empty() {
                        active_ids.extend(collect_active_elements(
                            reveals,
                            state,
                            all_elements,
                            "",
                        ));
                    }
                }
            }
            Element::Group { elements, when, .. } => {
                if is_visible(when.as_deref()) {
                    // Recursively collect from group
                    active_ids.extend(collect_active_elements(elements, state, all_elements, ""));
                }
            }
            Element::Text { id, when, .. } | Element::Markdown { id, when, .. } => {
                // Static elements are included in active list if visible and identified
                if is_visible(when.as_deref()) {
                    if let Some(text_id) = id {
                        active_ids.push(text_id.clone());
                    }
                }
            }
        }
    }

    active_ids
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn collect_active_elements_for_test(
        elements: &[Element],
        state: &PopupState,
        all_elements: &[Element],
    ) -> Vec<String> {
        super::collect_active_elements(elements, state, all_elements, "")
    }
}
