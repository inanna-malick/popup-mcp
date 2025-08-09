use anyhow::Result;
use eframe::egui;
use egui::{Context, CentralPanel, Layout, RichText, ScrollArea, Vec2, Stroke, Key, Id, Color32};
use egui_twemoji::EmojiLabel;
use std::sync::{Arc, Mutex};

use crate::models::{Element, PopupDefinition, PopupResult, PopupState, Condition, ComparisonOp};
use crate::theme::Theme;

mod widget_renderers;

fn setup_custom_fonts(ctx: &Context) {
    // Install image loaders for egui-twemoji (required for emoji rendering)
    egui_extras::install_image_loaders(ctx);
    log::info!("Installed image loaders for emoji support");
}


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
        Box::new(move |cc| {
            // Configure fonts for emoji support
            setup_custom_fonts(&cc.egui_ctx);
            
            let app = PopupApp::new_with_result(definition, result_clone);
            Ok(Box::new(app))
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
    first_interactive_widget_id: Option<Id>,
    first_widget_focused: bool,
}

impl PopupApp {
    fn new_with_result(definition: PopupDefinition, result: Arc<Mutex<Option<PopupResult>>>) -> Self {
        let state = PopupState::new(&definition);
        Self {
            definition,
            state,
            theme: Theme::soft_focus(),
            result,
            first_interactive_widget_id: None,
            first_widget_focused: false,
        }
    }
    
    
    fn send_result_and_close(&mut self, ctx: &Context) {
        let popup_result = PopupResult::from_state_with_context(&self.state, &self.definition);
        *self.result.lock().unwrap() = Some(popup_result);
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

impl eframe::App for PopupApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Apply theme
        self.theme.apply_to_egui(ctx);
        
        // Handle Escape key for cancel
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.state.button_clicked = Some("Cancel".to_string());
        }
        
        // Check if we should close
        if self.state.button_clicked.is_some() {
            self.send_result_and_close(ctx);
            return;
        }
        
        CentralPanel::default().show(ctx, |ui| {
            // Extremely compact for no-scroll layout
            ui.spacing_mut().item_spacing = Vec2::new(4.0, 1.0);
            ui.spacing_mut().button_padding = Vec2::new(6.0, 3.0);
            ui.spacing_mut().indent = 8.0;  // Minimal indentation
            
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    render_elements_with_context(
                        ui, 
                        &self.definition.elements, 
                        &mut self.state, 
                        &self.definition.elements, 
                        &self.theme,
                        &mut self.first_interactive_widget_id,
                        self.first_widget_focused
                    );
                });
        });
        
        // Check again after rendering in case a button was clicked
        if self.state.button_clicked.is_some() {
            self.send_result_and_close(ctx);
        }
        
        // Request focus on first interactive widget if not already focused
        if !self.first_widget_focused {
            if let Some(widget_id) = self.first_interactive_widget_id {
                ctx.memory_mut(|mem| mem.request_focus(widget_id));
                self.first_widget_focused = true;
            }
        }
    }
}

fn render_elements_with_context(
    ui: &mut egui::Ui,
    elements: &[Element],
    state: &mut PopupState,
    all_elements: &[Element],
    theme: &Theme,
    first_widget_id: &mut Option<Id>,
    widget_focused: bool,
) {
    let mut is_first = true;
    
    for element in elements {
        match element {
            Element::Text { content: text } => {
                if is_first && text.to_uppercase() == *text {
                    // Header style with cyberpunk glow
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.add_space(2.0);
                        let header_text = RichText::new(format!("▶ {}", text))
                            .size(16.0)
                            .color(theme.neon_cyan);
                        EmojiLabel::new(header_text).show(ui);
                    });
                    ui.add_space(2.0);
                    
                    // Neon separator
                    ui.separator();
                    ui.add_space(4.0);
                } else {
                    ui.horizontal(|ui| {
                        ui.add_space(10.0);
                        EmojiLabel::new(RichText::new(text).color(theme.text_primary)).show(ui);
                    });
                }
                is_first = false;
            }
            
            Element::Slider { label, min, max, default: _ } => {
                if let Some(value) = state.get_number_mut(label) {
                    ui.horizontal(|ui| {
                        // Compact inline display: "Energy: [slider] 4/10"
                        ui.label(RichText::new(format!("{}:", label)).color(theme.text_primary));
                        
                        // Slider takes most of the space
                        let slider = egui::Slider::new(value, *min..=*max)
                            .show_value(false)
                            .clamping(egui::SliderClamping::Always);
                        let response = ui.add(slider);
                        
                        // Simple value display
                        ui.label(RichText::new(format!("{}/{}", *value as i32, *max as i32))
                            .color(theme.text_secondary)
                            .monospace());
                        
                        // Store the response ID for focus
                        if first_widget_id.is_none() && !widget_focused {
                            *first_widget_id = Some(response.id);
                        }
                    });
                }
            }
            
            Element::Checkbox { label, default: _ } => {
                if let Some(value) = state.get_boolean_mut(label) {
                    ui.horizontal(|ui| {
                        
                        let checkbox_text = if *value {
                            RichText::new(format!("☑ {}", label)).color(theme.text_primary).strong()
                        } else {
                            RichText::new(format!("☐ {}", label)).color(theme.text_primary)
                        };
                        let response = ui.selectable_label(false, checkbox_text);
                        if response.clicked() {
                            *value = !*value;
                        }
                        
                        // Store the response ID for focus
                        if first_widget_id.is_none() && !widget_focused {
                            *first_widget_id = Some(response.id);
                        }
                        
                        // Make checkbox focusable with keyboard
                        if response.has_focus() && ui.input(|i| i.key_pressed(Key::Space)) {
                            *value = !*value;
                        }
                    });
                }
            }
            
            Element::Textbox { label, placeholder, rows } => {
                ui.group(|ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(label).color(theme.text_primary).strong());
                        if let Some(value) = state.get_text(label) {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.small(format!("{} chars", value.len()));
                            });
                        }
                    });
                    if let Some(value) = state.get_text_mut(label) {
                        let height = rows.unwrap_or(1) as f32 * 20.0;
                        let text_edit = egui::TextEdit::multiline(value)
                            .desired_width(ui.available_width())
                            .min_size(Vec2::new(ui.available_width(), height));
                        
                        let response = if let Some(hint) = placeholder {
                            ui.add(text_edit.hint_text(hint))
                        } else {
                            ui.add(text_edit)
                        };
                        
                        // Store the response ID for focus
                        if first_widget_id.is_none() && !widget_focused {
                            *first_widget_id = Some(response.id);
                        }
                    }
                });
            }
            
            Element::Choice { label, options, .. } => {
                ui.group(|ui| {
                    ui.set_width(ui.available_width());
                    ui.label(RichText::new(label).color(theme.text_primary).strong());
                    if let Some(selected) = state.get_choice_mut(label) {
                        ui.vertical(|ui| {
                            // Handle arrow key navigation
                            let up_pressed = ui.input(|i| i.key_pressed(Key::ArrowUp));
                            let down_pressed = ui.input(|i| i.key_pressed(Key::ArrowDown));
                            let has_focus = options.iter().enumerate().any(|(i, _)| {
                                ui.ctx().memory(|mem| mem.has_focus(egui::Id::new(format!("choice_{}_{}", label, i))))
                            });
                            
                            if has_focus {
                                if up_pressed && *selected > 0 {
                                    *selected -= 1;
                                } else if down_pressed && *selected < options.len() - 1 {
                                    *selected += 1;
                                }
                            }
                            
                            for (i, option) in options.iter().enumerate() {
                                let is_selected = *selected == i;
                                let option_text = if is_selected {
                                    RichText::new(format!("▸ {}", option)).color(theme.text_primary).strong()
                                } else {
                                    RichText::new(format!("  {}", option)).color(theme.text_primary)
                                };
                                let response = ui.selectable_label(is_selected, option_text)
                                    .on_hover_text(format!("Option {} of {}", i + 1, options.len()));
                                if response.clicked() {
                                    *selected = i;
                                }
                                
                                // Store the response ID for focus (only for first item)
                                if first_widget_id.is_none() && !widget_focused && i == 0 {
                                    *first_widget_id = Some(response.id);
                                }
                                
                                // Make selectable with keyboard
                                if response.has_focus() && ui.input(|i| i.key_pressed(Key::Space) || i.key_pressed(Key::Enter)) {
                                    *selected = i;
                                }
                            }
                        });
                    }
                });
            }
            
            Element::Multiselect { label, options } => {
                ui.group(|ui| {
                    ui.set_width(ui.available_width());
                    if let Some(selections) = state.get_multichoice_mut(label) {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(label).color(theme.text_primary).strong());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let selected_count = selections.iter().filter(|&&x| x).count();
                                if selected_count > 0 {
                                    ui.small(format!("{} selected", selected_count));
                                }
                            });
                        });
                        // Add Select All/None buttons
                        ui.horizontal(|ui| {
                            if ui.small_button("All").clicked() {
                                selections.iter_mut().for_each(|s| *s = true);
                            }
                            if ui.small_button("None").clicked() {
                                selections.iter_mut().for_each(|s| *s = false);
                            }
                        });
                        // Use horizontal layout with two columns for 6 items
                        if options.len() > 4 {
                            ui.horizontal(|ui| {
                                let mid = options.len() / 2;
                                
                                // First column
                                ui.vertical(|ui| {
                                    for i in 0..mid {
                                        if i < selections.len() {
                                            let checkbox_text = if selections[i] {
                                                RichText::new(format!("☑ {}", options[i])).color(theme.text_primary).strong()
                                            } else {
                                                RichText::new(format!("☐ {}", options[i])).color(theme.text_primary)
                                            };
                                            let response = ui.selectable_label(false, checkbox_text);
                                            if response.clicked() {
                                                selections[i] = !selections[i];
                                            }
                                            if first_widget_id.is_none() && !widget_focused && i == 0 {
                                                *first_widget_id = Some(response.id);
                                            }
                                        }
                                    }
                                });
                                
                                ui.add_space(20.0); // Space between columns
                                
                                // Second column
                                ui.vertical(|ui| {
                                    for i in mid..options.len() {
                                        if i < selections.len() {
                                            let checkbox_text = if selections[i] {
                                                RichText::new(format!("☑ {}", options[i])).color(theme.text_primary).strong()
                                            } else {
                                                RichText::new(format!("☐ {}", options[i])).color(theme.text_primary)
                                            };
                                            let response = ui.selectable_label(false, checkbox_text);
                                            if response.clicked() {
                                                selections[i] = !selections[i];
                                            }
                                        }
                                    }
                                });
                            });
                        } else {
                            // Single column for 4 or fewer items
                            ui.vertical(|ui| {
                                for (i, option) in options.iter().enumerate() {
                                    if i < selections.len() {
                                        let checkbox_text = if selections[i] {
                                            RichText::new(format!("☑ {}", option)).color(theme.text_primary).strong()
                                        } else {
                                            RichText::new(format!("☐ {}", option)).color(theme.text_primary)
                                        };
                                        let response = ui.selectable_label(false, checkbox_text);
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
                    }
                });
            }
            
            Element::Group { label, elements } => {
                ui.group(|ui| {
                    ui.label(RichText::new(label).color(theme.text_primary).strong());
                    ui.separator();
                    render_elements_with_context(ui, elements, state, all_elements, theme, first_widget_id, widget_focused);
                });
            }
            
            Element::Conditional { condition, elements } => {
                if evaluate_condition_with_context(condition, state, all_elements) {
                    render_elements_with_context(ui, elements, state, all_elements, theme, first_widget_id, widget_focused);
                }
            }
            
            Element::Buttons { labels: buttons } => {
                ui.separator();
                ui.add_space(4.0);
                
                ui.with_layout(Layout::left_to_right(egui::Align::Center), |ui| {
                    // Calculate dynamic button width based on text content
                    let max_text_len = buttons.iter().map(|b| b.len()).max().unwrap_or(8);
                    let button_width = (max_text_len as f32 * 7.0 + 20.0).min(120.0).max(80.0);
                    let total_width = buttons.len() as f32 * (button_width + 8.0);
                    let available_width = ui.available_width();
                    if available_width > total_width {
                        ui.add_space((available_width - total_width) / 2.0);
                    }
                    
                    for (i, button) in buttons.iter().enumerate() {
                        
                        let button_text = RichText::new(button.to_uppercase())
                            .size(12.0)
                            .strong()
                            .color(Color32::BLACK);
                        
                        let button_color = if button.contains("Force") || button.contains("Cancel") {
                            theme.neon_pink
                        } else if button.contains("Continue") || button.contains("Proceed") {
                            theme.matrix_green
                        } else {
                            theme.electric_blue
                        };
                        
                        // For buttons, we need to use a different approach since Button expects a WidgetText
                        let response = if button.chars().any(|c| c as u32 > 0x7F) {
                            // Contains non-ASCII characters (likely emoji)
                            // First, render the emoji and get its rect
                            let emoji_response = ui.allocate_ui_with_layout(
                                Vec2::new(button_width, 28.0),
                                Layout::centered_and_justified(egui::Direction::LeftToRight),
                                |ui| {
                                    EmojiLabel::new(button_text).show(ui);
                                }
                            );
                            
                            // Then create a clickable overlay at the same position
                            let response = ui.interact(
                                emoji_response.response.rect,
                                egui::Id::new(format!("emoji_button_{}", i)),
                                egui::Sense::click()
                            );
                            
                            // Draw translucent button overlay
                            if ui.is_rect_visible(response.rect) {
                                let visuals = ui.style().interact(&response);
                                ui.painter().rect(
                                    response.rect,
                                    visuals.corner_radius,
                                    button_color.linear_multiply(0.1), // More translucent
                                    Stroke::new(1.0, button_color),
                                    egui::StrokeKind::Middle
                                );
                            }
                            
                            response
                        } else {
                            // Regular text button
                            ui.add_sized(
                                Vec2::new(button_width, 28.0),
                                egui::Button::new(button_text)
                                    .fill(button_color.linear_multiply(0.2))
                                    .stroke(Stroke::new(1.0, button_color))
                            )
                        };
                        
                        // Store the response ID for focus (only for first button)
                        if first_widget_id.is_none() && !widget_focused && i == 0 {
                            *first_widget_id = Some(response.id);
                        }
                        
                        if response.clicked() {
                            state.button_clicked = Some(button.clone());
                        }
                        
                        // Handle Enter key on focused button
                        if response.has_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
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
        Condition::Simple(name) | Condition::Checked { checked: name } => {
            state.get_boolean(name)
        }
        Condition::Selected { selected: label, value: expected_value } => {
            let selected_idx = state.get_choice(label);
            if let Some(actual_value) = find_selected_option(all_elements, label, selected_idx) {
                actual_value == *expected_value
            } else {
                false
            }
        }
        Condition::Count { count: label, op, value } => {
            if let Some(selections) = state.get_multichoice(label) {
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
            Element::Choice { label, options, .. } if label == choice_name => {
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
    let mut height: f32 = 35.0; // Title bar with some padding
    let mut max_width: f32 = 350.0; // Reasonable default width
    
    calculate_elements_size(&definition.elements, &mut height, &mut max_width, 0, true);
    
    // Add some bottom padding
    height += 15.0;
    
    // Reasonable bounds for complex UIs
    max_width = max_width.min(450.0).max(320.0);  // Wide enough for columns
    height = height.min(600.0);  // Should fit on most screens
    
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
            Element::Text { content: text } => {
                *height += 20.0; // Compact text
                *max_width = max_width.max(text.len() as f32 * 7.0 + 20.0);
            }
            Element::Slider { label, .. } => {
                *height += 20.0; // Very compact slider line
                *max_width = max_width.max(label.len() as f32 * 7.0 + 150.0);
            }
            Element::Checkbox { label, .. } => {
                *height += 18.0; // Tiny checkbox
                *max_width = max_width.max(label.len() as f32 * 7.0 + 50.0);
            }
            Element::Textbox { rows, .. } => {
                *height += 20.0 + 18.0 * (*rows).unwrap_or(1) as f32; // Minimal textbox
                *max_width = max_width.max(320.0);
            }
            Element::Choice { options, .. } => {
                *height += 20.0; // Label
                *height += 18.0 * options.len() as f32; // Tiny radio buttons
                let longest = options.iter().map(|s| s.len()).max().unwrap_or(0);
                *max_width = max_width.max((longest as f32) * 7.0 + 60.0);
            }
            Element::Multiselect { options, .. } => {
                *height += 20.0; // Label
                *height += 20.0; // All/None buttons
                // If using columns, height is reduced
                if options.len() > 4 {
                    let rows_per_column = (options.len() + 1) / 2;
                    *height += 18.0 * rows_per_column as f32;
                    *max_width = max_width.max(300.0); // Need more width for columns
                } else {
                    *height += 18.0 * options.len() as f32;
                    let longest = options.iter().map(|s| s.len()).max().unwrap_or(0);
                    *max_width = max_width.max((longest as f32) * 8.0 + 100.0);
                }
            }
            Element::Group { elements, .. } => {
                *height += 18.0; // Tiny group header
                calculate_elements_size(elements, height, max_width, depth + 1, include_conditionals);
                *height += 2.0; // Almost no group padding
            }
            Element::Conditional { elements, condition } => {
                if include_conditionals {
                    // Use probability heuristic
                    let probability = match condition {
                        Condition::Selected { .. } => 0.7,
                        Condition::Simple(_) | Condition::Checked { .. } => 0.3,
                        Condition::Count { .. } => 0.2,
                    };
                    
                    let start_height = *height;
                    calculate_elements_size(elements, height, max_width, depth, include_conditionals);
                    let added_height = *height - start_height;
                    *height = start_height + (added_height * probability);
                }
            }
            Element::Buttons { labels: buttons } => {
                *height += 28.0; // Small button row
                let button_width = buttons.len() as f32 * 70.0; // Narrower buttons
                *max_width = max_width.max(button_width + 15.0); // Tight margins
            }
        }
        *height += 1.0; // Almost no item spacing
    }
}