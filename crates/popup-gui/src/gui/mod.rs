use anyhow::Result;
use eframe::egui;
use egui::{CentralPanel, Context, Id, Key, RichText, ScrollArea, TopBottomPanel, Vec2};
use std::sync::{Arc, Mutex};

use crate::theme::Theme;
use popup_common::{Condition, Element, PopupDefinition, PopupResult, PopupState};


#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn collect_active_elements_for_test(
        elements: &[Element],
        state: &PopupState,
        all_elements: &[Element],
    ) -> Vec<String> {
        super::collect_active_elements(elements, state, all_elements)
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
            ui.add_space(5.0);
            ui.separator();
            ui.add_space(5.0);
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                // Use a simpler approach - just text without RichText formatting
                if ui.button("SUBMIT").clicked() {
                    self.state.button_clicked = Some("submit".to_string());
                    self.send_result_and_close(ctx);
                }
            });
            ui.add_space(5.0);
        });

        // Main content in central panel
        CentralPanel::default().show(ctx, |ui| {
            // Extremely compact for no-scroll layout
            ui.spacing_mut().item_spacing = Vec2::new(4.0, 1.0);
            ui.spacing_mut().button_padding = Vec2::new(6.0, 3.0);
            ui.spacing_mut().indent = 8.0; // Minimal indentation

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // Try grid layout for the entire form
                    render_elements_in_grid(
                        ui,
                        &self.definition.elements,
                        &mut self.state,
                        &self.definition.elements,
                        &self.theme,
                        &mut self.first_interactive_widget_id,
                        self.first_widget_focused,
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

fn render_elements_in_grid(
    ui: &mut egui::Ui,
    elements: &[Element],
    state: &mut PopupState,
    all_elements: &[Element],
    theme: &Theme,
    first_widget_id: &mut Option<Id>,
    widget_focused: bool,
) {
    // Render all elements in order with proper vertical layout
    ui.vertical(|ui| {
        for element in elements.iter() {
            render_single_element(
                ui,
                element,
                state,
                all_elements,
                theme,
                first_widget_id,
                widget_focused,
            );

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
    theme: &Theme,
    first_widget_id: &mut Option<Id>,
    widget_focused: bool,
) {
    match element {
        Element::Text { content: text } => {
            ui.label(RichText::new(text).color(theme.text_primary));
        }

        Element::Multiselect { label, options } => {
            ui.group(|ui| {
                // Clone selections to avoid borrow conflict when rendering conditionals
                let selections_snapshot = if let Some(selections) = state.get_multichoice_mut(label)
                {
                    ui.label(RichText::new(label).color(theme.matrix_green).strong());
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
                                        .color(theme.matrix_green)
                                        .strong()
                                } else {
                                    RichText::new(format!("☐ {}", option.value()))
                                        .color(theme.text_primary)
                                };
                                let response = col.selectable_label(false, checkbox_text);
                                if response.clicked() {
                                    selections[i] = !selections[i];
                                }
                                if first_widget_id.is_none() && !widget_focused && i == 0 {
                                    *first_widget_id = Some(response.id);
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
                        if let Some(children) = option.conditional() {
                            ui.indent(format!("multiselect_cond_{}_{}", label, i), |ui| {
                                render_elements_in_grid(
                                    ui,
                                    children,
                                    state,
                                    all_elements,
                                    theme,
                                    first_widget_id,
                                    widget_focused,
                                );
                            });
                        }
                    }
                }
            });
        }

        Element::Choice { label, options, .. } => {
            ui.label(RichText::new(label).color(theme.text_primary));
            if let Some(selected) = state.get_choice_mut(label) {
                let selected_text = match *selected {
                    Some(idx) => options
                        .get(idx)
                        .map(|opt| opt.value())
                        .unwrap_or("(invalid)"),
                    None => "(none selected)",
                };

                egui::ComboBox::from_id_salt(label)
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        // Option to clear selection
                        if ui
                            .selectable_label(selected.is_none(), "(none selected)")
                            .clicked()
                        {
                            *selected = None;
                        }
                        // Show all options
                        for (idx, option) in options.iter().enumerate() {
                            if ui
                                .selectable_label(*selected == Some(idx), option.value())
                                .clicked()
                            {
                                *selected = Some(idx);
                            }
                        }
                    });

                // Render inline conditional for selected option
                if let Some(idx) = *selected {
                    if let Some(opt) = options.get(idx) {
                        if let Some(children) = opt.conditional() {
                            ui.indent(format!("choice_cond_{}_{}", label, idx), |ui| {
                                render_elements_in_grid(
                                    ui,
                                    children,
                                    state,
                                    all_elements,
                                    theme,
                                    first_widget_id,
                                    widget_focused,
                                );
                            });
                        }
                    }
                }
            }
            ui.add_space(6.0);
        }

        Element::Checkbox {
            label, conditional, ..
        } => {
            if let Some(value) = state.get_boolean_mut(label) {
                let checkbox_text = if *value {
                    RichText::new(format!("☑ {}", label))
                        .color(theme.matrix_green)
                        .strong()
                } else {
                    RichText::new(format!("☐ {}", label)).color(theme.text_primary)
                };
                let response = ui.selectable_label(false, checkbox_text);
                if response.clicked() {
                    *value = !*value;
                }

                // Render inline conditional if checkbox is checked
                if *value {
                    if let Some(children) = conditional {
                        ui.indent(format!("checkbox_cond_{}", label), |ui| {
                            render_elements_in_grid(
                                ui,
                                children,
                                state,
                                all_elements,
                                theme,
                                first_widget_id,
                                widget_focused,
                            );
                        });
                    }
                }
            }
        }

        Element::Slider {
            label,
            min,
            max,
            default: _,
        } => {
            ui.group(|ui| {
                ui.label(RichText::new(label).color(theme.warning_orange).strong());
                if let Some(value) = state.get_number_mut(label) {
                    ui.horizontal(|ui| {
                        let slider = egui::Slider::new(value, *min..=*max)
                            .show_value(false)
                            .clamping(egui::SliderClamping::Always);
                        let response = ui.add(slider);
                        ui.label(
                            RichText::new(format!("{:.1}/{:.1}", *value, *max))
                                .color(theme.text_secondary)
                                .text_style(egui::TextStyle::Small),
                        );

                        if first_widget_id.is_none() && !widget_focused {
                            *first_widget_id = Some(response.id);
                        }
                    });
                }
            });
        }

        Element::Textbox {
            label,
            placeholder,
            rows,
        } => {
            ui.group(|ui| {
                ui.label(RichText::new(label).color(theme.neon_purple).strong());
                if let Some(value) = state.get_text_mut(label) {
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

        Element::Group { label, elements } => {
            ui.group(|ui| {
                ui.label(RichText::new(label).color(theme.neon_pink).strong());
                ui.add_space(4.0);
                render_elements_in_grid(
                    ui,
                    elements,
                    state,
                    all_elements,
                    theme,
                    first_widget_id,
                    widget_focused,
                );
            });
        }

        Element::Conditional {
            condition,
            elements,
        } => {
            // Check if condition is met using shared helper
            let show = evaluate_condition(condition, state, all_elements);

            if show {
                render_elements_in_grid(
                    ui,
                    elements,
                    state,
                    all_elements,
                    theme,
                    first_widget_id,
                    widget_focused,
                );
            }
        }
    }
}

// Helper functions

/// Collect only the active elements based on current state (evaluating conditionals)
fn collect_active_elements(
    elements: &[Element],
    state: &PopupState,
    all_elements: &[Element],
) -> Vec<String> {
    let mut active_labels = Vec::new();

    for element in elements {
        match element {
            Element::Slider { label, .. } | Element::Textbox { label, .. } => {
                active_labels.push(label.clone());
            }
            Element::Checkbox {
                label, conditional, ..
            } => {
                active_labels.push(label.clone());
                // If checkbox is checked and has inline conditional, collect from it
                if state.get_boolean(label) {
                    if let Some(children) = conditional {
                        active_labels.extend(collect_active_elements(
                            children,
                            state,
                            all_elements,
                        ));
                    }
                }
            }
            Element::Multiselect { label, options } => {
                active_labels.push(label.clone());
                // For each checked option with conditional, collect from it
                if let Some(selections) = state.get_multichoice(label) {
                    for (i, option) in options.iter().enumerate() {
                        if i < selections.len() && selections[i] {
                            if let Some(children) = option.conditional() {
                                active_labels.extend(collect_active_elements(
                                    children,
                                    state,
                                    all_elements,
                                ));
                            }
                        }
                    }
                }
            }
            Element::Choice { label, options, .. } => {
                active_labels.push(label.clone());
                // If there's a selected option with conditional, collect from it
                if let Some(Some(idx)) = state.get_choice(label) {
                    if let Some(option) = options.get(idx) {
                        if let Some(children) = option.conditional() {
                            active_labels.extend(collect_active_elements(
                                children,
                                state,
                                all_elements,
                            ));
                        }
                    }
                }
            }
            Element::Group { elements, .. } => {
                // Recursively collect from group
                active_labels.extend(collect_active_elements(elements, state, all_elements));
            }
            Element::Conditional {
                condition,
                elements,
            } => {
                // Only collect from conditional if condition is met
                if evaluate_condition(condition, state, all_elements) {
                    active_labels.extend(collect_active_elements(elements, state, all_elements));
                }
            }
            Element::Text { .. } => {
                // Text elements don't have state
            }
        }
    }

    active_labels
}

/// Evaluate if a condition is met based on current state
fn evaluate_condition(condition: &Condition, state: &PopupState, all_elements: &[Element]) -> bool {
    match condition {
        Condition::Simple(label) => {
            // Pattern 1: Check if checkbox is true OR any multiselect option is selected OR choice has selection
            if state.get_boolean(label) {
                true // Checkbox is checked
            } else if let Some(selections) = state.get_multichoice(label) {
                selections.iter().any(|&selected| selected) // Any multiselect option selected
            } else if let Some(Some(_)) = state.get_choice(label) {
                true // Choice has a selection
            } else {
                false
            }
        }
        Condition::Field { field, value } => {
            // Pattern 2: Check if checkbox name matches value OR multiselect has specific option selected OR choice has specific option selected
            if state.get_boolean(field) && field == value {
                true // Checkbox checked and field name matches value
            } else if let Some(selections) = state.get_multichoice(field) {
                // Find multiselect options and check if the specified value is selected
                if let Some(options) = find_multiselect_options(all_elements, field) {
                    options
                        .iter()
                        .position(|opt| opt == value)
                        .and_then(|index| selections.get(index))
                        .copied()
                        .unwrap_or(false)
                } else {
                    false
                }
            } else if let Some(Some(idx)) = state.get_choice(field) {
                // Find choice options and check if the selected option matches value
                if let Some(options) = find_choice_options(all_elements, field) {
                    options.get(idx).map(|s| s == value).unwrap_or(false)
                } else {
                    false
                }
            } else {
                false
            }
        }
        Condition::Count { field, count } => {
            // Pattern 3: Count-based conditions
            use popup_common::ComparisonOp;

            if let Some((op, target_value)) = ComparisonOp::parse_count_condition(count) {
                let actual_count = if state.get_boolean(field) {
                    1 // Checkbox counts as 1 if checked, 0 if not
                } else if let Some(selections) = state.get_multichoice(field) {
                    selections.iter().filter(|&&x| x).count() as i32
                } else if let Some(choice) = state.get_choice(field) {
                    if choice.is_some() {
                        1
                    } else {
                        0
                    } // Choice counts as 1 if selected, 0 if not
                } else {
                    0
                };

                match op {
                    ComparisonOp::Greater => actual_count > target_value,
                    ComparisonOp::Less => actual_count < target_value,
                    ComparisonOp::GreaterEqual => actual_count >= target_value,
                    ComparisonOp::LessEqual => actual_count <= target_value,
                    ComparisonOp::Equal => actual_count == target_value,
                }
            } else {
                false // Invalid count condition
            }
        }
    }
}

fn find_multiselect_options(elements: &[Element], label: &str) -> Option<Vec<String>> {
    for element in elements {
        match element {
            Element::Multiselect {
                label: el_label,
                options,
            } if el_label == label => {
                return Some(options.iter().map(|opt| opt.value().to_string()).collect());
            }
            Element::Group { elements, .. } | Element::Conditional { elements, .. } => {
                // Recursively search in nested elements
                if let Some(options) = find_multiselect_options(elements, label) {
                    return Some(options);
                }
            }
            _ => {}
        }
    }
    None
}

fn find_choice_options(elements: &[Element], label: &str) -> Option<Vec<String>> {
    for element in elements {
        match element {
            Element::Choice {
                label: el_label,
                options,
                ..
            } if el_label == label => {
                return Some(options.iter().map(|opt| opt.value().to_string()).collect());
            }
            Element::Group { elements, .. } | Element::Conditional { elements, .. } => {
                // Recursively search in nested elements
                if let Some(options) = find_choice_options(elements, label) {
                    return Some(options);
                }
            }
            _ => {}
        }
    }
    None
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
            Element::Text { content: text } => {
                *height += 40.0; // Moderately larger text requires more height
                *max_width = max_width.max(text.len() as f32 * 12.0 + 40.0); // Moderately larger character width
            }
            Element::Slider { label, .. } => {
                if uses_slider_grid {
                    // For grid layout: need width for 2 columns + spacing with larger text
                    // Each column needs: label + slider + value display
                    *max_width = max_width.max(550.0); // More space for grid layout with moderately larger text
                }
                *height += 50.0; // Moderately larger slider height for bigger text and spacing
                *max_width = max_width.max(label.len() as f32 * 12.0 + 220.0); // Moderately larger character width + slider
            }
            Element::Checkbox { label, .. } => {
                *height += 35.0; // Moderately larger checkbox height for bigger text
                *max_width = max_width.max(label.len() as f32 * 12.0 + 90.0); // Moderately larger character width + checkbox
            }
            Element::Textbox { rows, .. } => {
                *height += 35.0 + 30.0 * (*rows).unwrap_or(1) as f32; // Moderately larger textbox height per row
                *max_width = max_width.max(420.0); // More width for text input with moderately larger font
            }
            Element::Multiselect { options, .. } => {
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
            }
            Element::Choice { label, options, .. } => {
                *height += 35.0; // Label height
                *height += 35.0; // ComboBox height
                let longest = options
                    .iter()
                    .map(|opt| opt.value().len())
                    .max()
                    .unwrap_or(0)
                    .max(label.len());
                *max_width = max_width.max((longest as f32) * 12.0 + 100.0); // Character width + dropdown indicator
            }
            Element::Group { elements, .. } => {
                *height += 40.0; // Moderately larger group header height for bigger text
                calculate_elements_size(
                    elements,
                    height,
                    max_width,
                    include_conditionals,
                );
                *height += 15.0; // Proper group padding
            }
            Element::Conditional {
                elements,
                condition,
            } => {
                if include_conditionals {
                    // Use probability heuristic
                    let probability = match condition {
                        Condition::Simple(_) => 0.3,
                        Condition::Field { .. } => 0.4,
                        Condition::Count { .. } => 0.2,
                    };

                    let start_height = *height;
                    calculate_elements_size(
                        elements,
                        height,
                        max_width,
                        include_conditionals,
                    );
                    let added_height = *height - start_height;
                    *height = start_height + (added_height * probability);
                }
            }
        }
        *height += 5.0; // Proper item spacing
    }
}
