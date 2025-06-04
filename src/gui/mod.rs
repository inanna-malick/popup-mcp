use anyhow::Result;
use eframe::egui;
use egui::{Context, CentralPanel, Layout, RichText, ScrollArea, Vec2};
use std::sync::{Arc, Mutex};

use crate::models::{Element, PopupDefinition, PopupResult, PopupState, Condition, ComparisonOp};
use crate::theme::Theme;

mod widget_renderers;

pub fn render_popup(definition: PopupDefinition) -> Result<PopupResult> {
    use std::sync::{Arc, Mutex};
    
    // Calculate initial window size
    let (width, height) = calculate_window_size(&definition);
    
    let result = Arc::new(Mutex::new(None));
    let result_clone = result.clone();
    
    let title = definition.title.clone();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([width, height])
            .with_resizable(false)
            .with_position(egui::Pos2::new(100.0, 100.0)),  // Will center manually if needed
        ..Default::default()
    };
    
    eframe::run_native(
        &title,
        options,
        Box::new(move |_cc| {
            let app = PopupApp::new_with_result(definition, result_clone);
            Box::new(app)
        }),
    ).map_err(|e| anyhow::anyhow!("Failed to run eframe: {}", e))?;
    
    // Extract result
    let result = result.lock().unwrap().take()
        .ok_or_else(|| anyhow::anyhow!("Popup closed without result"))?;
    
    Ok(result)
}

struct PopupApp {
    definition: PopupDefinition,
    state: PopupState,
    theme: Theme,
    result: Arc<Mutex<Option<PopupResult>>>,
}

impl PopupApp {
    fn new_with_result(definition: PopupDefinition, result: Arc<Mutex<Option<PopupResult>>>) -> Self {
        let state = PopupState::new(&definition);
        Self {
            definition,
            state,
            theme: Theme::spike_neural(),
            result,
        }
    }
    
    fn send_result_and_close(&mut self, ctx: &Context) {
        let popup_result = PopupResult::from_state(&self.state);
        *self.result.lock().unwrap() = Some(popup_result);
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

impl eframe::App for PopupApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Apply theme
        self.theme.apply_to_egui(ctx);
        
        // Check if we should close
        if self.state.button_clicked.is_some() {
            self.send_result_and_close(ctx);
            return;
        }
        
        CentralPanel::default().show(ctx, |ui| {
            // Add minimal padding
            ui.spacing_mut().item_spacing = Vec2::new(4.0, 2.0);
            ui.spacing_mut().button_padding = Vec2::new(6.0, 2.0);
            
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    render_elements_with_context(ui, &self.definition.elements, &mut self.state, &self.definition.elements, &self.theme);
                });
        });
        
        // Check again after rendering in case a button was clicked
        if self.state.button_clicked.is_some() {
            self.send_result_and_close(ctx);
        }
    }
}

fn render_elements_with_context(
    ui: &mut egui::Ui,
    elements: &[Element],
    state: &mut PopupState,
    all_elements: &[Element],
    theme: &Theme,
) {
    let mut is_first = true;
    
    for element in elements {
        match element {
            Element::Text(text) => {
                if is_first && text.to_uppercase() == *text {
                    // Header style
                    ui.colored_label(theme.neural_blue, RichText::new(text).size(14.0));
                    ui.separator();
                } else {
                    ui.label(text);
                }
                is_first = false;
            }
            
            Element::Slider { label, min, max, default: _ } => {
                if let Some(value) = state.sliders.get_mut(label) {
                    ui.horizontal(|ui| {
                        ui.label(label);
                        ui.add(egui::Slider::new(value, *min..=*max));
                    });
                }
            }
            
            Element::Checkbox { label, default: _ } => {
                if let Some(value) = state.checkboxes.get_mut(label) {
                    ui.checkbox(value, label);
                }
            }
            
            Element::Textbox { label, placeholder, rows } => {
                ui.label(label);
                if let Some(value) = state.textboxes.get_mut(label) {
                    let height = rows.unwrap_or(1) as f32 * 20.0;
                    if let Some(hint) = placeholder {
                        ui.add_sized(
                            Vec2::new(ui.available_width(), height),
                            egui::TextEdit::multiline(value).hint_text(hint)
                        );
                    } else {
                        ui.add_sized(
                            Vec2::new(ui.available_width(), height),
                            egui::TextEdit::multiline(value)
                        );
                    }
                }
            }
            
            Element::Choice { label, options } => {
                ui.label(label);
                if let Some(selected) = state.choices.get_mut(label) {
                    ui.vertical(|ui| {
                        for (i, option) in options.iter().enumerate() {
                            ui.radio_value(selected, i, option);
                        }
                    });
                }
            }
            
            Element::Multiselect { label, options } => {
                ui.label(label);
                if let Some(selections) = state.multiselects.get_mut(label) {
                    ui.vertical(|ui| {
                        for (i, option) in options.iter().enumerate() {
                            if i < selections.len() {
                                ui.checkbox(&mut selections[i], option);
                            }
                        }
                    });
                }
            }
            
            Element::Group { label, elements } => {
                ui.group(|ui| {
                    ui.label(RichText::new(label).strong());
                    render_elements_with_context(ui, elements, state, all_elements, theme);
                });
            }
            
            Element::Conditional { condition, elements } => {
                if evaluate_condition_with_context(condition, state, all_elements) {
                    render_elements_with_context(ui, elements, state, all_elements, theme);
                }
            }
            
            Element::Buttons(buttons) => {
                ui.separator();
                ui.add_space(4.0);
                
                ui.with_layout(Layout::left_to_right(egui::Align::Center), |ui| {
                    // Center the buttons
                    let button_width = 90.0;
                    let total_width = buttons.len() as f32 * (button_width + 8.0);
                    let available_width = ui.available_width();
                    if available_width > total_width {
                        ui.add_space((available_width - total_width) / 2.0);
                    }
                    
                    for button in buttons {
                        let response = ui.add_sized(
                            Vec2::new(button_width, 26.0),
                            egui::Button::new(button)
                                .fill(theme.neural_blue)
                        );
                        
                        if response.clicked() {
                            state.button_clicked = Some(button.clone());
                        }
                    }
                });
            }
        }
    }
}

fn evaluate_condition_with_context(
    condition: &Condition,
    state: &PopupState,
    all_elements: &[Element]
) -> bool {
    match condition {
        Condition::Checked(name) => {
            state.checkboxes.get(name).copied().unwrap_or(false)
        }
        Condition::Selected(name, expected_value) => {
            if let Some(&selected_idx) = state.choices.get(name) {
                if let Some(actual_value) = find_selected_option(all_elements, name, selected_idx) {
                    actual_value == *expected_value
                } else {
                    false
                }
            } else {
                false
            }
        }
        Condition::Count(name, op, value) => {
            if let Some(selections) = state.multiselects.get(name) {
                let count = selections.iter().filter(|&&x| x).count() as i32;
                match op {
                    ComparisonOp::Greater => count > *value,
                    ComparisonOp::Less => count < *value,
                    ComparisonOp::GreaterEqual => count >= *value,
                    ComparisonOp::LessEqual => count <= *value,
                    ComparisonOp::Equal => count == *value,
                }
            } else {
                false
            }
        }
    }
}

fn find_selected_option(elements: &[Element], choice_name: &str, selected_idx: usize) -> Option<String> {
    for element in elements {
        match element {
            Element::Choice { label, options } if label == choice_name => {
                return options.get(selected_idx).cloned();
            }
            Element::Group { elements, .. } | Element::Conditional { elements, .. } => {
                if let Some(found) = find_selected_option(elements, choice_name, selected_idx) {
                    return Some(found);
                }
            }
            _ => {}
        }
    }
    None
}

fn calculate_window_size(definition: &PopupDefinition) -> (f32, f32) {
    let mut height: f32 = 35.0; // Title bar
    let mut max_width: f32 = 350.0; // Minimum width
    
    calculate_elements_size(&definition.elements, &mut height, &mut max_width, 0, true);
    
    // Add bottom padding
    height += 10.0;
    
    // Set reasonable bounds
    max_width = max_width.min(550.0).max(350.0);
    height = height.min(800.0);
    
    (max_width, height)
}

fn calculate_elements_size(
    elements: &[Element],
    height: &mut f32,
    max_width: &mut f32,
    depth: usize,
    include_conditionals: bool,
) {
    for element in elements {
        match element {
            Element::Text(text) => {
                *height += 20.0;
                *max_width = max_width.max(text.len() as f32 * 7.0 + 20.0 + (depth as f32 * 10.0));
            }
            Element::Slider { label, .. } => {
                *height += 30.0;
                *max_width = max_width.max(label.len() as f32 * 7.0 + 200.0 + (depth as f32 * 10.0));
            }
            Element::Checkbox { label, .. } => {
                *height += 22.0;
                *max_width = max_width.max(label.len() as f32 * 7.0 + 60.0 + (depth as f32 * 10.0));
            }
            Element::Textbox { rows, .. } => {
                *height += 22.0 + 20.0 * (*rows).unwrap_or(1) as f32;
                *max_width = max_width.max(380.0 + (depth as f32 * 15.0));
            }
            Element::Choice { options, .. } => {
                *height += 20.0; // Label
                *height += 22.0 * options.len() as f32; // Options
                let longest = options.iter().map(|s| s.len()).max().unwrap_or(0);
                *max_width = max_width.max((longest as f32) * 7.0 + 60.0 + (depth as f32 * 10.0));
            }
            Element::Multiselect { options, .. } => {
                *height += 20.0; // Label
                *height += 22.0 * options.len() as f32; // Options
                let longest = options.iter().map(|s| s.len()).max().unwrap_or(0);
                *max_width = max_width.max((longest as f32) * 7.0 + 60.0 + (depth as f32 * 10.0));
            }
            Element::Group { elements, .. } => {
                *height += 30.0; // Group header and padding
                calculate_elements_size(elements, height, max_width, depth + 1, include_conditionals);
                *height += 10.0; // Bottom padding
            }
            Element::Conditional { elements, condition } => {
                if include_conditionals {
                    // Use probability heuristic
                    let probability = match condition {
                        Condition::Selected(_, _) => 0.7,
                        Condition::Checked(_) => 0.3,
                        Condition::Count(_, _, _) => 0.2,
                    };
                    
                    let start_height = *height;
                    calculate_elements_size(elements, height, max_width, depth, include_conditionals);
                    let added_height = *height - start_height;
                    *height = start_height + (added_height * probability);
                }
            }
            Element::Buttons(buttons) => {
                *height += 35.0;
                let button_width = buttons.len() as f32 * 98.0;
                *max_width = max_width.max(button_width);
            }
        }
        *height += 2.0; // Item spacing
    }
}