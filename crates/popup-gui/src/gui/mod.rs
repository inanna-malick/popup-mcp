use anyhow::Result;
use eframe::egui;
use egui::{CentralPanel, Context, Id, Key, Rect, RichText, ScrollArea, TopBottomPanel, Vec2};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use std::sync::{Arc, Mutex};

use crate::theme::Theme;
use popup_common::{evaluate_condition, parse_condition};
use popup_common::{Element, PopupDefinition, PopupResult, PopupState};

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

fn setup_custom_fonts(ctx: &Context) {
    // Install image loaders for egui-twemoji (required for emoji rendering)
    egui_extras::install_image_loaders(ctx);

    // Configure moderately larger text sizes (40% increase = 1.4x multiplier)
    let mut style = (*ctx.style()).clone();

    // Increase all text styles by ~40%
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(20.0, egui::FontFamily::Proportional), // was ~14.5, now 20
    );
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(17.0, egui::FontFamily::Proportional), // was ~12, now 17
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(15.0, egui::FontFamily::Proportional), // was ~11, now 15
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::new(13.0, egui::FontFamily::Proportional), // was ~9, now 13
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::new(22.0, egui::FontFamily::Monospace), // was ~10, now 22
    );

    ctx.set_style(style);
    log::info!("Installed image loaders for emoji support and configured larger text sizes");
}

pub fn render_popup(definition: PopupDefinition) -> Result<PopupResult> {
    use std::sync::{Arc, Mutex};

    let result = Arc::new(Mutex::new(None));
    let result_clone = result.clone();

    let title = definition.effective_title().to_string();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            // Start with a default size, it will be resized on the first frame
            .with_inner_size([400.0, 200.0])
            // Allow the window to be resized by the user
            .with_resizable(true)
            .with_position(egui::Pos2::new(100.0, 100.0)), // Will center manually if needed
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
        }
    }

    fn send_result_and_close(&mut self, ctx: &Context) {
        // Collect only active element labels based on current state
        let active_labels = collect_active_elements(
            &self.definition.elements,
            &self.state,
            &self.definition.elements,
            "",
        );

        let popup_result = PopupResult::from_state_with_active_elements(
            &self.state,
            &self.definition,
            &active_labels,
        );
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

        // Render the bottom panel and get its height
        let bottom_panel_response = TopBottomPanel::bottom("submit_panel").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                let button_text = RichText::new("SUBMIT")
                    .size(18.0)
                    .strong()
                    .color(self.theme.text_primary);
                let button = egui::Button::new(button_text)
                    .min_size(egui::Vec2::new(120.0, 40.0))
                    .fill(self.theme.electric_blue.linear_multiply(0.3));

                if ui.add(button).clicked() {
                    self.state.button_clicked = Some("submit".to_string());
                }
            });
            ui.add_space(8.0);
        });
        let bottom_panel_height = bottom_panel_response.response.rect.height();

        // Render the main content and measure its size
        CentralPanel::default().show(ctx, |ui| {
            // Improved spacing for better readability
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 6.0);
            ui.spacing_mut().button_padding = Vec2::new(10.0, 6.0);
            ui.spacing_mut().indent = 12.0;

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // Use a scope to measure the content rect
                    let content_response = ui.scope(|ui| {
                        let mut render_ctx = RenderContext {
                            theme: &self.theme,
                            first_widget_id: &mut self.first_interactive_widget_id,
                            widget_focused: self.first_widget_focused,
                        };
                        render_elements_in_grid(
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
                        mem.data
                            .insert_temp("content_rect".into(), content_response.response.rect)
                    });
                });
        });

        // --- Phase 2: Calculate Desired Size and Resize ---

        // Retrieve the content rect from memory
        let content_rect = ctx
            .memory(|mem| mem.data.get_temp::<Rect>("content_rect".into()))
            .unwrap_or(Rect::ZERO);

        let desired_width = content_rect.width() + ctx.style().spacing.window_margin.sum().x;
        let desired_height = content_rect.height()
            + bottom_panel_height
            + ctx.style().spacing.window_margin.sum().y
            + 5.0; // Add a small buffer to prevent scrollbar appearing unnecessarily

        // Clamp size to reasonable limits
        let new_size = Vec2::new(
            desired_width.clamp(400.0, 700.0),
            desired_height.clamp(200.0, 800.0),
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

// Removed old rendering functions that are no longer used

struct RenderContext<'a> {
    theme: &'a Theme,
    first_widget_id: &'a mut Option<Id>,
    widget_focused: bool,
}

fn render_elements_in_grid(
    ui: &mut egui::Ui,
    elements: &[Element],
    state: &mut PopupState,
    all_elements: &[Element],
    ctx: &mut RenderContext,
    path_prefix: &str,
) {
    // Render all elements in order with proper vertical layout
    ui.vertical(|ui| {
        for (idx, element) in elements.iter().enumerate() {
            let element_path = if path_prefix.is_empty() {
                idx.to_string()
            } else {
                format!("{}.{}", path_prefix, idx)
            };

            render_single_element(ui, element, state, all_elements, ctx, &element_path);

            // Force line break after each element
            ui.end_row();
        }
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
    let when_clause = match element {
        Element::Text { when, .. } => when,
        Element::Markdown { when, .. } => when,
        Element::Slider { when, .. } => when,
        Element::Check { when, .. } => when,
        Element::Input { when, .. } => when,
        Element::Multi { when, .. } => when,
        Element::Select { when, .. } => when,
        Element::Group { when, .. } => when,
    };

    if let Some(when_expr) = when_clause {
        let state_values = state.to_value_map(all_elements);
        match parse_condition(when_expr) {
            Ok(ast) => {
                if !evaluate_condition(&ast, &state_values) {
                    // Condition not met - don't render this element
                    return;
                }
            }
            Err(e) => {
                // Log warning but render anyway (fail-open)
                log::warn!("Failed to parse when clause '{}': {}", when_expr, e);
            }
        }
    }

    match element {
        Element::Text { text, .. } => {
            // Use element path as unique ID to prevent collisions in conditionals
            ui.push_id(format!("text_{}", element_path), |ui| {
                ui.label(RichText::new(text).color(ctx.theme.text_primary));
            });
        }

        Element::Markdown { markdown, .. } => {
            // Use element path as unique ID to prevent collisions in conditionals
            ui.push_id(format!("markdown_{}", element_path), |ui| {
                // Create a temporary cache for this render - caching across frames
                // would require storing in app state, but this works for simple cases
                let mut cache = CommonMarkCache::default();
                CommonMarkViewer::new().show(ui, &mut cache, markdown);
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
            let widget_frame = egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::same(10))
                .stroke(egui::Stroke::new(
                    1.0,
                    ctx.theme.matrix_green.linear_multiply(0.3),
                ));

            widget_frame.show(ui, |ui| {
                // Clone selections to avoid borrow conflict when rendering conditionals
                let selections_snapshot = if let Some(selections) = state.get_multichoice_mut(id) {
                    ui.label(
                        RichText::new(multi)
                            .color(ctx.theme.matrix_green)
                            .strong()
                            .size(15.0),
                    );
                    ui.horizontal(|ui| {
                        if ui.small_button("All").clicked() {
                            selections.iter_mut().for_each(|s| *s = true);
                        }
                        if ui.small_button("None").clicked() {
                            selections.iter_mut().for_each(|s| *s = false);
                        }
                    });

                    // Use 3-column layout for 6 items
                    ui.columns(3, |cols| {
                        for (i, option) in options.iter().enumerate() {
                            let col = &mut cols[i % 3];
                            if i < selections.len() {
                                let checkbox_text = if selections[i] {
                                    RichText::new(format!("☑ {}", option.value()))
                                        .color(ctx.theme.matrix_green)
                                        .strong()
                                } else {
                                    RichText::new(format!("☐ {}", option.value()))
                                        .color(ctx.theme.text_primary)
                                };
                                // Show description as tooltip if present
                                let response = col.selectable_label(false, checkbox_text);
                                if let Some(desc) = option.description() {
                                    response.clone().on_hover_text(desc);
                                }
                                if response.clicked() {
                                    selections[i] = !selections[i];
                                }
                                if ctx.first_widget_id.is_none() && !ctx.widget_focused && i == 0 {
                                    *ctx.first_widget_id = Some(response.id);
                                }
                            }
                        }
                    });

                    selections.clone()
                } else {
                    vec![]
                };

                // Render inline conditionals for each checked option (after borrow is dropped)
                for (i, option) in options.iter().enumerate() {
                    if i < selections_snapshot.len() && selections_snapshot[i] {
                        if let Some(children) = option_children.get(option.value()) {
                            ui.indent(format!("multiselect_cond_{}_{}", id, i), |ui| {
                                render_elements_in_grid(
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

                // Render reveals if multiselect has any AND any option is selected
                let has_selection = selections_snapshot.iter().any(|&s| s);
                if has_selection && !reveals.is_empty() {
                    ui.indent(format!("multiselect_reveals_{}", id), |ui| {
                        render_elements_in_grid(
                            ui,
                            reveals,
                            state,
                            all_elements,
                            ctx,
                            element_path,
                        );
                    });
                }
            });
        }

        Element::Select {
            select,
            id,
            options,
            option_children,
            reveals,
            ..
        } => {
            ui.label(RichText::new(select).color(ctx.theme.text_primary));

            // Clone selection state to avoid borrow conflict
            let selected_option = state.get_choice_mut(id).and_then(|s| *s);

            if let Some(selected) = state.get_choice_mut(id) {
                let selected_text = match *selected {
                    Some(idx) => options.get(idx).map(|s| s.value()).unwrap_or("(invalid)"),
                    None => "(none selected)",
                };

                egui::ComboBox::from_id_salt(id)
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        // Option to clear selection
                        if ui
                            .selectable_label(selected.is_none(), "(none selected)")
                            .clicked()
                        {
                            *selected = None;
                        }
                        // Show all options with descriptions as tooltips
                        for (idx, option) in options.iter().enumerate() {
                            let response =
                                ui.selectable_label(*selected == Some(idx), option.value());
                            if let Some(desc) = option.description() {
                                response.clone().on_hover_text(desc);
                            }
                            if response.clicked() {
                                *selected = Some(idx);
                            }
                        }
                    });
            }

            // Render inline conditional for selected option (after borrow is dropped)
            if let Some(idx) = selected_option {
                if let Some(option_val) = options.get(idx) {
                    if let Some(children) = option_children.get(option_val.value()) {
                        ui.indent(format!("choice_cond_{}_{}", id, idx), |ui| {
                            render_elements_in_grid(
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

            // Render reveals if choice has any AND an option is selected
            if selected_option.is_some() && !reveals.is_empty() {
                ui.indent(format!("choice_reveals_{}", id), |ui| {
                    render_elements_in_grid(ui, reveals, state, all_elements, ctx, element_path);
                });
            }

            ui.add_space(6.0);
        }

        Element::Check {
            check, id, reveals, ..
        } => {
            if let Some(value) = state.get_boolean_mut(id) {
                let checkbox_text = if *value {
                    RichText::new(format!("☑ {}", check))
                        .color(ctx.theme.matrix_green)
                        .strong()
                } else {
                    RichText::new(format!("☐ {}", check)).color(ctx.theme.text_primary)
                };
                let response = ui.selectable_label(false, checkbox_text);
                if response.clicked() {
                    *value = !*value;
                }

                // Render reveals if checkbox is checked
                if *value && !reveals.is_empty() {
                    ui.indent(format!("checkbox_reveals_{}", id), |ui| {
                        render_elements_in_grid(
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
            ..
        } => {
            let widget_frame = egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::same(10))
                .stroke(egui::Stroke::new(
                    1.0,
                    ctx.theme.warning_orange.linear_multiply(0.3),
                ));

            widget_frame.show(ui, |ui| {
                ui.label(
                    RichText::new(slider)
                        .color(ctx.theme.warning_orange)
                        .strong()
                        .size(15.0),
                );
                if let Some(value) = state.get_number_mut(id) {
                    ui.horizontal(|ui| {
                        let available_width = ui.available_width();
                        let value_label_width = 70.0; // Reserve space for "X.X/Y.Y" display
                        let slider_width = (available_width - value_label_width).max(150.0);

                        let slider = egui::Slider::new(value, *min..=*max)
                            .show_value(false)
                            .clamping(egui::SliderClamping::Always)
                            .min_decimals(1)
                            .max_decimals(1);

                        ui.spacing_mut().slider_width = slider_width;
                        let response = ui.add(slider);

                        ui.label(
                            RichText::new(format!("{:.1}/{:.1}", *value, *max))
                                .color(ctx.theme.text_secondary)
                                .text_style(egui::TextStyle::Small),
                        );

                        if ctx.first_widget_id.is_none() && !ctx.widget_focused {
                            *ctx.first_widget_id = Some(response.id);
                        }
                    });
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
            let widget_frame = egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::same(10))
                .stroke(egui::Stroke::new(
                    1.0,
                    ctx.theme.neon_purple.linear_multiply(0.3),
                ));

            widget_frame.show(ui, |ui| {
                ui.label(
                    RichText::new(input)
                        .color(ctx.theme.neon_purple)
                        .strong()
                        .size(15.0),
                );
                if let Some(value) = state.get_text_mut(id) {
                    let height = rows.unwrap_or(1) as f32 * 20.0;
                    let text_edit = egui::TextEdit::multiline(value)
                        .desired_width(ui.available_width())
                        .min_size(Vec2::new(ui.available_width(), height));

                    if let Some(hint) = placeholder {
                        ui.add(text_edit.hint_text(hint));
                    } else {
                        ui.add(text_edit);
                    }
                }
            });
        }

        Element::Group {
            group, elements, ..
        } => {
            // Enhanced group with better visual separation
            let group_frame = egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::same(12))
                .stroke(egui::Stroke::new(
                    1.5,
                    ctx.theme.electric_blue.linear_multiply(0.4),
                ));

            group_frame.show(ui, |ui| {
                ui.label(
                    RichText::new(group)
                        .color(ctx.theme.neon_pink)
                        .strong()
                        .size(16.0),
                );
                ui.add_space(8.0);
                render_elements_in_grid(
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
    let is_visible = |when: &Option<String>| -> bool {
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
                if is_visible(when) {
                    active_ids.push(id.clone());
                }
            }
            Element::Check {
                id, reveals, when, ..
            } => {
                if is_visible(when) {
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
                if is_visible(when) {
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
                if is_visible(when) {
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
                if is_visible(when) {
                    // Recursively collect from group
                    active_ids.extend(collect_active_elements(elements, state, all_elements, ""));
                }
            }
            Element::Text { id, when, .. } => {
                // Text elements are included in active list if visible
                if is_visible(when) {
                    if let Some(text_id) = id {
                        active_ids.push(text_id.clone());
                    }
                }
            }
            Element::Markdown { id, when, .. } => {
                // Markdown elements are included in active list if visible
                if is_visible(when) {
                    if let Some(md_id) = id {
                        active_ids.push(md_id.clone());
                    }
                }
            }
        }
    }

    active_ids
}


