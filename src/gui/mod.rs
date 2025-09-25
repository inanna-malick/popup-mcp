use anyhow::Result;
use eframe::egui;
use egui::{CentralPanel, Context, Id, Key, RichText, ScrollArea, TopBottomPanel, Vec2};
use std::sync::{Arc, Mutex};

use crate::models::{Condition, Element, PopupDefinition, PopupResult, PopupState};
use crate::theme::Theme;

mod widget_renderers;

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

/// Render popups sequentially. Due to GUI event loop limitations, only one popup
/// can be shown at a time. Additional popups will wait until the previous one closes.
///
/// IMPORTANT: On macOS, the first popup MUST be created from the main thread.
/// Consider using render_popup() directly from the main thread instead.
#[cfg(feature = "async")]
pub async fn render_popup_sequential(definition: PopupDefinition) -> Result<PopupResult> {
    // Use a global queue to ensure popups show one at a time
    static POPUP_QUEUE: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    // Acquire the lock - this ensures only one popup shows at a time
    let _guard = POPUP_QUEUE.lock().await;

    // For now, just call the synchronous version
    // This will block the current thread but ensures proper event loop handling
    render_popup(definition)
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
                        "main",
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
    grid_id_suffix: &str,
) {
    // Render all elements in order with proper vertical layout
    ui.vertical(|ui| {
        for (_idx, element) in elements.iter().enumerate() {
            render_single_element(
                ui,
                element,
                state,
                all_elements,
                theme,
                first_widget_id,
                widget_focused,
                grid_id_suffix,
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
    _grid_id_suffix: &str,
) {
    match element {
        Element::Text { content: text } => {
            ui.label(RichText::new(text).color(theme.text_primary));
        }

        Element::Choice { label, options, .. } => {
            ui.group(|ui| {
                ui.label(RichText::new(label).color(theme.electric_blue).strong());
                if let Some(selected) = state.get_choice_mut(label) {
                    for (i, option) in options.iter().enumerate() {
                        let is_selected = *selected == i;
                        let option_text = if is_selected {
                            RichText::new(format!("▸ {}", option))
                                .color(theme.neon_cyan)
                                .strong()
                        } else {
                            RichText::new(format!("  {}", option)).color(theme.text_primary)
                        };
                        let response = ui.selectable_label(is_selected, option_text);
                        if response.clicked() {
                            *selected = i;
                        }
                    }
                }
            });
        }

        Element::Multiselect { label, options } => {
            ui.group(|ui| {
                if let Some(selections) = state.get_multichoice_mut(label) {
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
                                    RichText::new(format!("☑ {}", option))
                                        .color(theme.matrix_green)
                                        .strong()
                                } else {
                                    RichText::new(format!("☐ {}", option)).color(theme.text_primary)
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
                }
            });
        }

        Element::Checkbox { label, .. } => {
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
            }
        }

        Element::Slider { label, min, max, default: _ } => {
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
                    &format!("group_{}", label),
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
                // Create unique ID based on condition type
                let cond_id = match condition {
                    Condition::Simple(label) => format!("cond_{}", label),
                    Condition::Checked { checked } => format!("cond_{}", checked),
                    Condition::Selected { selected, .. } => format!("cond_sel_{}", selected),
                    Condition::Count { count, .. } => format!("cond_cnt_{}", count),
                };
                render_elements_in_grid(
                    ui,
                    elements,
                    state,
                    all_elements,
                    theme,
                    first_widget_id,
                    widget_focused,
                    &cond_id,
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
            Element::Slider { label, .. }
            | Element::Checkbox { label, .. }
            | Element::Textbox { label, .. }
            | Element::Choice { label, .. }
            | Element::Multiselect { label, .. } => {
                active_labels.push(label.clone());
            }
            Element::Group { elements, .. } => {
                // Recursively collect from group
                active_labels.extend(collect_active_elements(elements, state, all_elements));
            }
            Element::Conditional { condition, elements } => {
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
            // Check if a checkbox with this label is true
            state.get_boolean(label)
        }
        Condition::Checked { checked } => {
            // Check if a checkbox with this label is true
            state.get_boolean(checked)
        }
        Condition::Selected { selected, value } => {
            // Check if a choice with this label has the specified value selected
            let selected_idx = state.get_choice(selected);

            // Find the Choice element with matching label to get its options
            if let Some(options) = find_choice_options(all_elements, selected) {
                options
                    .get(selected_idx)
                    .map(|selected_option| selected_option == value)
                    .unwrap_or(false)
            } else {
                false
            }
        }
        Condition::Count { count, value, op } => {
            // Count selected items in a multiselect
            if let Some(selections) = state.get_multichoice(count) {
                let selected_count = selections.iter().filter(|&&x| x).count();
                use crate::models::ComparisonOp;
                match op {
                    ComparisonOp::Greater => selected_count > *value as usize,
                    ComparisonOp::Less => selected_count < *value as usize,
                    ComparisonOp::GreaterEqual => selected_count >= *value as usize,
                    ComparisonOp::LessEqual => selected_count <= *value as usize,
                    ComparisonOp::Equal => selected_count == *value as usize,
                }
            } else {
                false
            }
        }
    }
}

fn find_choice_options(elements: &[Element], label: &str) -> Option<Vec<String>> {
    for element in elements {
        match element {
            Element::Choice {
                label: el_label,
                options,
                ..
            } if el_label == label => {
                return Some(options.clone());
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

    calculate_elements_size(&definition.elements, &mut height, &mut max_width, 0, true);

    // Add space for the Submit button panel (separator + button + padding)
    height += 70.0; // TopBottomPanel with Submit button and spacing

    // Add base padding for window chrome and margins
    height += 20.0; // Additional margin
    max_width += 40.0; // Side margins

    // Reasonable bounds for complex UIs
    // Allow wider windows for slider grids (need ~420px for 2 columns)
    max_width = max_width.min(700.0).max(400.0); // Increased minimum and maximum
    height = height.min(800.0); // Allow taller windows

    (max_width, height)
}

fn calculate_elements_size(
    elements: &[Element],
    height: &mut f32,
    max_width: &mut f32,
    depth: usize,
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
            Element::Choice { options, .. } => {
                *height += 35.0; // Moderately larger label with proper spacing
                *height += 30.0 * options.len() as f32; // Moderately larger radio button height
                let longest = options.iter().map(|s| s.len()).max().unwrap_or(0);
                *max_width = max_width.max((longest as f32) * 12.0 + 110.0); // Moderately larger character width + radio buttons
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
                    let longest = options.iter().map(|s| s.len()).max().unwrap_or(0);
                    *max_width = max_width.max((longest as f32) * 12.0 + 130.0); // Moderately larger character width + more space
                }
            }
            Element::Group { elements, .. } => {
                *height += 40.0; // Moderately larger group header height for bigger text
                calculate_elements_size(
                    elements,
                    height,
                    max_width,
                    depth + 1,
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
                        Condition::Selected { .. } => 0.7,
                        Condition::Simple(_) | Condition::Checked { .. } => 0.3,
                        Condition::Count { .. } => 0.2,
                    };

                    let start_height = *height;
                    calculate_elements_size(
                        elements,
                        height,
                        max_width,
                        depth,
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
