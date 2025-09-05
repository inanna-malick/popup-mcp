use anyhow::Result;
use eframe::egui;
use egui::{Context, CentralPanel, TopBottomPanel, RichText, ScrollArea, Vec2, Key, Id};
use std::sync::{Arc, Mutex};

use crate::models::{Element, PopupDefinition, PopupResult, PopupState, Condition};
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
                }
            });
            ui.add_space(5.0);
        });
        
        // Main content in central panel
        CentralPanel::default().show(ctx, |ui| {
            // Extremely compact for no-scroll layout
            ui.spacing_mut().item_spacing = Vec2::new(4.0, 1.0);
            ui.spacing_mut().button_padding = Vec2::new(6.0, 3.0);
            ui.spacing_mut().indent = 8.0;  // Minimal indentation
            
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
                        self.first_widget_focused
                    );
                });
        });
        
        // Check again after rendering in case submit was clicked
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
    use egui::Grid;
    
    // Collect sliders first
    let mut sliders = Vec::new();
    let mut other_elements = Vec::new();
    
    for element in elements {
        if matches!(element, Element::Slider { .. }) {
            sliders.push(element);
        } else {
            other_elements.push(element);
        }
    }
    
    // Render sliders in a 2x2 grid
    if !sliders.is_empty() {
        Grid::new("slider_grid")
            .num_columns(2)
            .spacing([10.0, 4.0])
            .min_col_width(200.0)
            .show(ui, |ui| {
                for (idx, slider) in sliders.iter().enumerate() {
                    if let Element::Slider { label, min, max, .. } = slider {
                        if let Some(value) = state.get_number_mut(label) {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("{}:", label)).color(theme.text_primary));
                                let slider = egui::Slider::new(value, *min..=*max)
                                    .show_value(false)
                                    .clamping(egui::SliderClamping::Always);
                                let response = ui.add(slider);
                                ui.label(RichText::new(format!("{}/{}", *value as i32, *max as i32))
                                    .color(theme.text_secondary)
                                    .monospace());
                                
                                if first_widget_id.is_none() && !widget_focused && idx == 0 {
                                    *first_widget_id = Some(response.id);
                                }
                            });
                            
                            // End row after every 2 sliders
                            if idx % 2 == 1 {
                                ui.end_row();
                            }
                        }
                    }
                }
            });
    }
    
    // Render other elements normally
    for element in other_elements {
        render_single_element(ui, element, state, all_elements, theme, first_widget_id, widget_focused);
    }
}

fn render_single_element(
    ui: &mut egui::Ui,
    element: &Element,
    state: &mut PopupState,
    _all_elements: &[Element],
    theme: &Theme,
    first_widget_id: &mut Option<Id>,
    widget_focused: bool,
) {
    match element {
        Element::Text { content: text } => {
            ui.label(RichText::new(text).color(theme.text_primary));
        }
        
        Element::Choice { label, options, .. } => {
            ui.group(|ui| {
                ui.label(RichText::new(label).color(theme.text_primary).strong());
                if let Some(selected) = state.get_choice_mut(label) {
                    for (i, option) in options.iter().enumerate() {
                        let is_selected = *selected == i;
                        let option_text = if is_selected {
                            RichText::new(format!("▸ {}", option)).color(theme.text_primary).strong()
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
                    ui.label(RichText::new(label).color(theme.text_primary).strong());
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
                                    RichText::new(format!("☑ {}", option)).color(theme.text_primary).strong()
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
                    RichText::new(format!("☑ {}", label)).color(theme.text_primary).strong()
                } else {
                    RichText::new(format!("☐ {}", label)).color(theme.text_primary)
                };
                let response = ui.selectable_label(false, checkbox_text);
                if response.clicked() {
                    *value = !*value;
                }
            }
        }
        
        Element::Textbox { label, placeholder, rows } => {
            ui.group(|ui| {
                ui.label(RichText::new(label).color(theme.text_primary).strong());
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
        
        _ => {}
    }
}

// Removed helper functions - conditionals not supported in grid layout yet

fn calculate_window_size(definition: &PopupDefinition) -> (f32, f32) {
    let mut height: f32 = 35.0; // Title bar with some padding
    let mut max_width: f32 = 350.0; // Reasonable default width
    
    calculate_elements_size(&definition.elements, &mut height, &mut max_width, 0, true);
    
    // Add space for the Submit button panel (separator + button + padding)
    height += 50.0; // TopBottomPanel with Submit button
    
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
        }
        *height += 1.0; // Almost no item spacing
    }
}