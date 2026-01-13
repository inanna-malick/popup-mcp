use anyhow::Result;
use eframe::egui;
use egui::{CentralPanel, Context, Id, Key, RichText, ScrollArea, TopBottomPanel, Vec2};
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

    // Calculate initial window size
    let (width, height) = calculate_window_size(&definition);

    let result = Arc::new(Mutex::new(None));
    let result_clone = result.clone();

    let title = definition.effective_title().to_string();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([width, height])
            .with_resizable(false)
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

        // Bottom panel for Submit button
        TopBottomPanel::bottom("submit_panel").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                // Prominent submit button with larger size and visual emphasis
                let button_text = RichText::new("SUBMIT")
                    .size(18.0)
                    .strong()
                    .color(self.theme.text_primary);
                let button = egui::Button::new(button_text)
                    .min_size(egui::Vec2::new(120.0, 40.0))
                    .fill(self.theme.electric_blue.linear_multiply(0.3));

                if ui.add(button).clicked() {
                    self.state.button_clicked = Some("submit".to_string());
                    self.send_result_and_close(ctx);
                }
            });
            ui.add_space(8.0);
        });

        // Main content in central panel
        CentralPanel::default().show(ctx, |ui| {
            // Improved spacing for better readability
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 6.0);
            ui.spacing_mut().button_padding = Vec2::new(10.0, 6.0);
            ui.spacing_mut().indent = 12.0;

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // Try grid layout for the entire form
                    let mut ctx = RenderContext {
                        theme: &self.theme,
                        first_widget_id: &mut self.first_interactive_widget_id,
                        widget_focused: self.first_widget_focused,
                    };
                    render_elements_in_grid(
                        ui,
                        &self.definition.elements,
                        &mut self.state,
                        &self.definition.elements,
                        &mut ctx,
                        "",
                    );
                });
        });

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

fn calculate_window_size(definition: &PopupDefinition) -> (f32, f32) {
    let mut height: f32 = 60.0; // Title bar with proper padding
    let mut max_width: f32 = 400.0; // More reasonable default width

    calculate_elements_size(&definition.elements, &mut height, &mut max_width, true);

    // Add space for the Submit button panel (separator + button + padding)
    height += 70.0; // TopBottomPanel with Submit button and spacing

    // Add base padding for window chrome and margins
    height += 20.0; // Additional margin
    max_width += 40.0; // Side margins

    // Reasonable bounds for complex UIs
    // Allow wider windows for slider grids (need ~420px for 2 columns)
    max_width = max_width.clamp(400.0, 700.0); // Increased minimum and maximum
    height = height.min(800.0); // Allow taller windows

    (max_width, height)
}

fn calculate_elements_size(
    elements: &[Element],
    height: &mut f32,
    max_width: &mut f32,
    include_conditionals: bool,
) {
    // Count sliders to see if we need grid layout
    let slider_count = elements
        .iter()
        .filter(|e| matches!(e, Element::Slider { .. }))
        .count();
    let uses_slider_grid = slider_count > 1;

    for element in elements {
        match element {
            Element::Text { text, .. } => {
                *height += 40.0; // Moderately larger text requires more height
                *max_width = max_width.max(text.len() as f32 * 12.0 + 40.0); // Moderately larger character width
            }
            Element::Markdown { markdown, .. } => {
                // Estimate height based on line count (rough approximation)
                let line_count = markdown.lines().count().max(1);
                *height += 25.0 * line_count as f32;
                // Estimate width based on longest line
                let longest_line = markdown.lines().map(|l| l.len()).max().unwrap_or(20);
                *max_width = max_width.max(longest_line as f32 * 10.0 + 40.0);
            }
            Element::Slider { slider, .. } => {
                if uses_slider_grid {
                    // For grid layout: need width for 2 columns + spacing with larger text
                    // Each column needs: label + slider + value display
                    *max_width = max_width.max(550.0); // More space for grid layout with moderately larger text
                }
                *height += 50.0; // Moderately larger slider height for bigger text and spacing
                *max_width = max_width.max(slider.len() as f32 * 12.0 + 220.0); // Moderately larger character width + slider
            }
            Element::Check { check, reveals, .. } => {
                *height += 35.0; // Moderately larger checkbox height for bigger text
                *max_width = max_width.max(check.len() as f32 * 12.0 + 90.0);
                // Moderately larger character width + checkbox

                // Include reveals in size calculation
                if include_conditionals && !reveals.is_empty() {
                    calculate_elements_size(reveals, height, max_width, include_conditionals);
                }
            }
            Element::Input { rows, .. } => {
                *height += 35.0 + 30.0 * (*rows).unwrap_or(1) as f32; // Moderately larger textbox height per row
                *max_width = max_width.max(420.0); // More width for text input with moderately larger font
            }
            Element::Multi { options, reveals, option_children, .. } => {
                *height += 35.0; // Moderately larger label with proper spacing
                *height += 40.0; // Moderately larger All/None buttons with spacing
                if options.len() > 4 {
                    let rows_per_column = options.len().div_ceil(2);
                    *height += 30.0 * rows_per_column as f32; // Moderately larger checkbox height
                    *max_width = max_width.max(500.0); // More width for columns with moderately larger text
                } else {
                    *height += 30.0 * options.len() as f32; // Moderately larger checkbox height
                    let longest = options
                        .iter()
                        .map(|opt| opt.value().len())
                        .max()
                        .unwrap_or(0);
                    *max_width = max_width.max((longest as f32) * 12.0 + 130.0);
                    // Moderately larger character width + more space
                }

                // Include reveals and option_children in size calculation
                if include_conditionals {
                    if !reveals.is_empty() {
                        calculate_elements_size(reveals, height, max_width, include_conditionals);
                    }
                    for children in option_children.values() {
                        calculate_elements_size(children, height, max_width, include_conditionals);
                    }
                }
            }
            Element::Select {
                select, options, reveals, option_children, ..
            } => {
                *height += 35.0; // Label height
                *height += 35.0; // ComboBox height
                let longest = options
                    .iter()
                    .map(|opt| opt.value().len())
                    .max()
                    .unwrap_or(0)
                    .max(select.len());
                *max_width = max_width.max((longest as f32) * 12.0 + 100.0); // Character width + dropdown indicator

                // Include reveals and option_children in size calculation
                if include_conditionals {
                    if !reveals.is_empty() {
                        calculate_elements_size(reveals, height, max_width, include_conditionals);
                    }
                    for children in option_children.values() {
                        calculate_elements_size(children, height, max_width, include_conditionals);
                    }
                }
            }
            Element::Group { elements, .. } => {
                *height += 40.0; // Moderately larger group header height for bigger text
                calculate_elements_size(elements, height, max_width, include_conditionals);
                *height += 15.0; // Proper group padding
            }
        }
        *height += 5.0; // Proper item spacing
    }
}
