use anyhow::Result;
use eframe::egui;
use egui::{CentralPanel, Color32, Context, Id, Key, Rect, RichText, ScrollArea, TopBottomPanel, Vec2};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::theme::Theme;
use popup_common::{evaluate_condition, parse_condition};
use popup_common::{ConditionExpr, Element, PopupDefinition, PopupResult, PopupState};

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
    
    // Start wider if we have multiple elements to encourage 2-column layout immediately
    let initial_size = if definition.elements.len() > 1 {
        [650.0, 400.0]
    } else {
        [400.0, 200.0]
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            // Start with a default size, it will be resized on the first frame
            .with_inner_size(initial_size)
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
    frame_count: usize,
    markdown_cache: CommonMarkCache,
    condition_cache: HashMap<String, Option<ConditionExpr>>,
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
            frame_count: 0,
            markdown_cache: CommonMarkCache::default(),
            condition_cache: HashMap::new(),
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
        self.frame_count += 1;

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
                    .color(self.theme.base2);
                let button = egui::Button::new(button_text)
                    .min_size(egui::Vec2::new(120.0, 40.0))
                    .fill(self.theme.neon_pink.linear_multiply(0.2));

                if ui.add(button).clicked() {
                    self.state.button_clicked = Some("submit".to_string());
                }
            });
            ui.add_space(8.0);
        });
        let bottom_panel_height = bottom_panel_response.response.rect.height();

        // Render the main content and measure its size
        CentralPanel::default()
            .show(ctx, |ui| {
            // Add outer margin manually using a frame
            egui::Frame::NONE
                .inner_margin(egui::Margin::same(10))
                .show(ui, |ui| {
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
                            markdown_cache: &mut self.markdown_cache,
                            condition_cache: &mut self.condition_cache,
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
        });

        // --- Phase 2: Calculate Desired Size and Resize ---

        // Retrieve the content rect from memory
        let content_rect = ctx
            .memory(|mem| mem.data.get_temp::<Rect>("content_rect".into()))
            .unwrap_or(Rect::ZERO);

        let chrome_w = ctx.style().spacing.window_margin.sum().x;
        let chrome_h = bottom_panel_height + ctx.style().spacing.window_margin.sum().y + 5.0;

        let desired_width = content_rect.width() + chrome_w;
        let desired_height = content_rect.height() + chrome_h;

        // Calculate a "preferred" width based on the complexity of the visible elements
        // This helps break the circular dependency where desired_width is constrained by current width.
        let visible_item_count = self.definition.elements.iter().filter(|e| {
             let when = match e {
                Element::Text { when, .. } => when,
                Element::Markdown { when, .. } => when,
                Element::Slider { when, .. } => when,
                Element::Check { when, .. } => when,
                Element::Input { when, .. } => when,
                Element::Multi { when, .. } => when,
                Element::Select { when, .. } => when,
                Element::Group { when, .. } => when,
            };
            if let Some(w) = when {
                let state_map = self.state.to_value_map(&self.definition.elements);
                parse_condition(w).map(|ast| evaluate_condition(&ast, &state_map)).unwrap_or(true)
            } else {
                true
            }
        }).count();

        let mut preferred_width = if visible_item_count > 1 {
            650.0
        } else {
            400.0
        };

        // If it's getting very tall, push the width out to encourage 2-column layout
        if desired_height > 500.0 && visible_item_count > 2 {
            preferred_width = 850.0;
        }

        // Get current window size (inner size)
        let current_rect = ctx.input(|i| i.viewport().inner_rect).unwrap_or(Rect::ZERO);
        let current_size = current_rect.size();

        let mut target_size = current_size;

        // Constraint 1: Min/Preferred width
        if target_size.x < preferred_width {
            target_size.x = preferred_width;
        }

        // Constraint 2: Expand to fit content (up to max)
        // Note: desired_width is often limited by current_size.x, but we still check it
        if desired_width > target_size.x {
            target_size.x = desired_width.min(1000.0);
        }
        if desired_height > target_size.y {
            target_size.y = desired_height.min(800.0);
        }

        // Constraint 3: Initial Shrink (snap to fit)
        if self.frame_count < 5 {
            target_size.x = desired_width.max(preferred_width).min(1000.0);
            target_size.y = desired_height.clamp(200.0, 800.0);
        }

        // Only issue command if significant change
        if (target_size - current_size).length_sq() > 1.0 {
            self.last_size = target_size;
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(target_size));
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
    condition_cache: &'a mut HashMap<String, Option<ConditionExpr>>,
}

fn render_elements_in_grid(
    ui: &mut egui::Ui,
    elements: &[Element],
    state: &mut PopupState,
    all_elements: &[Element],
    ctx: &mut RenderContext,
    path_prefix: &str,
) {
    let state_values = state.to_value_map(all_elements);

    // 1. Identify visible elements and their original indices
    let mut visible_indices = Vec::new();
    for (idx, element) in elements.iter().enumerate() {
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

        let is_visible = if let Some(when_expr) = when_clause {
            let cached_expr = ctx.condition_cache
                .entry(when_expr.clone())
                .or_insert_with(|| parse_condition(when_expr).ok());
            
            match cached_expr {
                Some(ast) => evaluate_condition(ast, &state_values),
                None => {
                    log::warn!("Failed to parse when clause: {}", when_expr);
                    true // fail-open
                },
            }
        } else {
            true
        };

        if is_visible {
            visible_indices.push(idx);
        }
    }

    if visible_indices.is_empty() {
        return;
    }

    // 2. Group consecutive simple checkboxes among visible elements
    let mut items = Vec::new();
    let mut i = 0;
    while i < visible_indices.len() {
        let idx = visible_indices[i];
        if let Element::Check { reveals, .. } = &elements[idx] {
            if reveals.is_empty() {
                let mut group = vec![idx];
                let mut next_i = i + 1;
                while next_i < visible_indices.len() {
                    let next_idx = visible_indices[next_i];
                    if let Element::Check { reveals, .. } = &elements[next_idx] {
                        if reveals.is_empty() {
                            group.push(next_idx);
                            next_i += 1;
                            continue;
                        }
                    }
                    break;
                }
                items.push(group);
                i = next_i;
                continue;
            }
        }
        items.push(vec![idx]);
        i += 1;
    }

    // 3. Render using columns if we have multiple items and enough space
    let available_width = ui.available_width();
    let use_columns = items.len() > 1 && available_width > 500.0;

    if use_columns {
        let mut left_items = Vec::new();
        let mut right_items = Vec::new();
        for (item_idx, item_indices) in items.into_iter().enumerate() {
            if item_idx % 2 == 0 {
                left_items.push(item_indices);
            } else {
                right_items.push(item_indices);
            }
        }

        ui.horizontal_top(|ui| {
            let total_width = ui.available_width();
            let gap = 32.0;
            let col_width = (total_width - gap) / 2.0;

            // Left Column
            let left_res = ui.allocate_ui_with_layout(
                egui::vec2(col_width, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    for item_indices in left_items {
                        render_item_group(
                            ui,
                            item_indices,
                            elements,
                            state,
                            all_elements,
                            ctx,
                            path_prefix,
                        );
                        ui.add_space(4.0);
                    }
                },
            );

            // Divider spacing
            let (sep_rect, _) = ui.allocate_at_least(egui::vec2(gap, 0.0), egui::Sense::hover());

            // Right Column
            let right_res = ui.allocate_ui_with_layout(
                egui::vec2(col_width, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    for item_indices in right_items {
                        render_item_group(
                            ui,
                            item_indices,
                            elements,
                            state,
                            all_elements,
                            ctx,
                            path_prefix,
                        );
                        ui.add_space(4.0);
                    }
                },
            );

            // Vertical Divider (Solarized Violet - IDE split style)
            let height = left_res.response.rect.height().max(right_res.response.rect.height());
            let center_x = sep_rect.center().x;
            let top_y = sep_rect.top();
            
            ui.painter().vline(
                center_x,
                top_y..=(top_y + height),
                egui::Stroke::new(1.0, ctx.theme.neon_purple),
            );
        });
    } else {
        ui.vertical(|ui| {
            for item_indices in items {
                render_item_group(
                    ui,
                    item_indices,
                    elements,
                    state,
                    all_elements,
                    ctx,
                    path_prefix,
                );
                ui.add_space(4.0);
            }
        });
    }
}

fn render_item_group(
    ui: &mut egui::Ui,
    item_indices: Vec<usize>,
    elements: &[Element],
    state: &mut PopupState,
    all_elements: &[Element],
    ctx: &mut RenderContext,
    path_prefix: &str,
) {
    let first_idx = item_indices[0];
    let is_simple_checkbox = matches!(&elements[first_idx], Element::Check { reveals, .. } if reveals.is_empty());

    if is_simple_checkbox {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 24.0;
            ui.spacing_mut().item_spacing.y = 8.0;
            for idx in item_indices {
                let element_path = if path_prefix.is_empty() {
                    idx.to_string()
                } else {
                    format!("{}.{}", path_prefix, idx)
                };
                render_single_element(ui, &elements[idx], state, all_elements, ctx, &element_path);
            }
        });
    } else {
        for idx in item_indices {
            let element_path = if path_prefix.is_empty() {
                idx.to_string()
            } else {
                format!("{}.{}", path_prefix, idx)
            };
            render_single_element(ui, &elements[idx], state, all_elements, ctx, &element_path);
        }
    }
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
        let cached_expr = ctx.condition_cache
            .entry(when_expr.clone())
            .or_insert_with(|| parse_condition(when_expr).ok());
        
        match cached_expr {
            Some(ast) => {
                if !evaluate_condition(ast, &state_values) {
                    // Condition not met - don't render this element
                    return;
                }
            },
            None => {
                // Log warning but render anyway (fail-open)
                log::warn!("Failed to parse when clause: {}", when_expr);
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
                ui.style_mut().visuals.override_text_color = Some(ctx.theme.neon_pink);
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
            // No widget frame for Minimalist approach
            ui.vertical(|ui| {
                let selections_snapshot = if let Some(selections) = state.get_multichoice_mut(id) {
                    ui.horizontal(|ui| {
                        let label_width = 140.0;
                        ui.add_sized(
                            [label_width, 24.0],
                            egui::Label::new(
                                RichText::new(multi)
                                    .color(ctx.theme.matrix_green)
                                    .strong()
                                    .size(15.0),
                            ),
                        );

                        ui.horizontal(|ui| {
                            if ui.button("Select All").clicked() {
                                selections.iter_mut().for_each(|s| *s = true);
                            }
                            if ui.button("Clear All").clicked() {
                                selections.iter_mut().for_each(|s| *s = false);
                            }
                        });
                    });

                    ui.add_space(4.0);

                    let available_width = ui.available_width();
                    let num_cols = if available_width >= 800.0 { 3 } else { 2 };

                    egui::Grid::new(format!("multi_grid_{}", id))
                        .num_columns(num_cols)
                        .spacing([20.0, 8.0])
                        .show(ui, |ui| {
                            for (i, option) in options.iter().enumerate() {
                                if i < selections.len() {
                                    // Constrain width for text wrapping
                                    ui.push_id(format!("multi_{}_{}", id, i), |ui| {
                                        ui.set_max_width(250.0);
                                        ui.horizontal_wrapped(|ui| {
                                            let mut value = selections[i];
                                            let response = ui.checkbox(&mut value, "");
                                            selections[i] = value;

                                            // Separate wrapped label
                                            ui.label(RichText::new(option.value()).color(ctx.theme.matrix_green));

                                            if let Some(desc) = option.description() {
                                                response.clone().on_hover_text(desc);
                                            }

                                            if ctx.first_widget_id.is_none() && !ctx.widget_focused && i == 0 {
                                                *ctx.first_widget_id = Some(response.id);
                                            }
                                        });
                                    });
                                }
                                // End row
                                if (i + 1) % num_cols == 0 {
                                    ui.end_row();
                                }
                            }
                        });

                    selections.clone()
                } else {
                    vec![]
                };
// ... (rest of Multi logic)
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
// ... (omitting Select for now as it's fine)
            select,
            id,
            options,
            option_children,
            reveals,
            ..
        } => {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    let label_width = 140.0;
                    ui.add_sized(
                        [label_width, 24.0],
                        egui::Label::new(RichText::new(select).color(ctx.theme.electric_blue).strong()),
                    );

                    if let Some(selected) = state.get_choice_mut(id) {
                        let selected_text = match *selected {
                            Some(idx) => options.get(idx).map(|s| s.value()).unwrap_or("(invalid)"),
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

                        if ctx.first_widget_id.is_none() && !ctx.widget_focused {
                            *ctx.first_widget_id = Some(response.response.id);
                        }
                    }
                });

                let selected_option = state.get_choice(id).flatten();
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

                if selected_option.is_some() && !reveals.is_empty() {
                    ui.indent(format!("choice_reveals_{}", id), |ui| {
                        render_elements_in_grid(ui, reveals, state, all_elements, ctx, element_path);
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

                if ctx.first_widget_id.is_none() && !ctx.widget_focused {
                    *ctx.first_widget_id = Some(response.id);
                }

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
                    let value_label_width = 80.0;
                    let slider_width = (available_width - value_label_width - 10.0).max(100.0);

                    ui.spacing_mut().slider_width = slider_width;
                    let slider_widget = egui::Slider::new(value, *min..=*max)
                        .show_value(false)
                        .clamping(egui::SliderClamping::Always)
                        .min_decimals(1)
                        .max_decimals(1);

                    let response = ui.add(slider_widget);

                    ui.label(
                        RichText::new(format!("{:.1}/{:.1}", *value, *max))
                            .color(ctx.theme.base2)
                            .text_style(egui::TextStyle::Small),
                    );

                    if ctx.first_widget_id.is_none() && !ctx.widget_focused {
                        *ctx.first_widget_id = Some(response.id);
                    }
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
            // Keep subtle sunken background for Inputs (from B)
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
                        let input_width = ui.available_width().min(600.0);
                        let text_edit = egui::TextEdit::multiline(value)
                            .text_color(ctx.theme.base2)
                            .desired_width(input_width)
                            .min_size(Vec2::new(input_width, height));

                        if let Some(hint) = placeholder {
                            ui.add(text_edit.hint_text(hint));
                        } else {
                            ui.add(text_edit);
                        }
                    }
                });
            });
        }

        Element::Group {
            group, elements, ..
        } => {
            // Minimal ghost frame for Groups (from B)
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
