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
            ui.push_id(format!("text_{}", element_path), |ui| {
                ui.label(RichText::new(text).color(ctx.theme.text_primary));
            });
        }

        Element::Markdown { markdown, .. } => {
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
            let widget_frame = egui::Frame::none()
                .inner_margin(egui::Margin::symmetric(8, 4))
                .fill(ctx.theme.dark_gray.linear_multiply(0.5))
                .stroke(egui::Stroke::new(1.0, ctx.theme.matrix_green.linear_multiply(0.2)));

            widget_frame.show(ui, |ui| {
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

                    egui::Grid::new(format!("multi_grid_{}", id))
                        .num_columns(3)
                        .spacing([20.0, 8.0])
                        .show(ui, |ui| {
                            for (i, option) in options.iter().enumerate() {
                                if i < selections.len() {
                                    let mut value = selections[i];
                                    let response = ui.checkbox(&mut value, option.value());
                                    selections[i] = value;

                                    if let Some(desc) = option.description() {
                                        response.clone().on_hover_text(desc);
                                    }

                                    if ctx.first_widget_id.is_none() && !ctx.widget_focused && i == 0 {
                                        *ctx.first_widget_id = Some(response.id);
                                    }
                                }
                                if (i + 1) % 3 == 0 {
                                    ui.end_row();
                                }
                            }
                        });

                    selections.clone()
                } else {
                    vec![]
                };

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
            select,
            id,
            options,
            option_children,
            reveals,
            ..
        } => {
            let widget_frame = egui::Frame::none()
                .inner_margin(egui::Margin::symmetric(8, 4))
                .fill(ctx.theme.dark_gray.linear_multiply(0.5))
                .stroke(egui::Stroke::new(1.0, ctx.theme.electric_blue.linear_multiply(0.2)));

            widget_frame.show(ui, |ui| {
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
        }

        Element::Check {
            check, id, reveals, ..
        } => {
            if let Some(value) = state.get_boolean_mut(id) {
                let widget_frame = egui::Frame::none()
                    .inner_margin(egui::Margin::symmetric(8, 4))
                    .fill(ctx.theme.dark_gray.linear_multiply(0.5))
                    .stroke(egui::Stroke::new(1.0, ctx.theme.matrix_green.linear_multiply(0.2)));

                widget_frame.show(ui, |ui| {
                    let check_text = RichText::new(check).color(ctx.theme.matrix_green).strong();
                    let response = ui.checkbox(value, check_text);

                    if ctx.first_widget_id.is_none() && !ctx.widget_focused {
                        *ctx.first_widget_id = Some(response.id);
                    }
                });

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
            let widget_frame = egui::Frame::none()
                .inner_margin(egui::Margin::symmetric(8, 4))
                .fill(ctx.theme.dark_gray.linear_multiply(0.5))
                .stroke(egui::Stroke::new(1.0, ctx.theme.warning_orange.linear_multiply(0.2)));

            widget_frame.show(ui, |ui| {
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
            });
        }

        Element::Input {
            input,
            id,
            placeholder,
            rows,
            ..
        } => {
            let widget_frame = egui::Frame::none()
                .inner_margin(egui::Margin::symmetric(8, 4))
                .fill(ctx.theme.dark_gray.linear_multiply(0.5))
                .stroke(egui::Stroke::new(1.0, ctx.theme.neon_purple.linear_multiply(0.2)));

            widget_frame.show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    let label_width = 140.0;
                    ui.add_sized(
                        [label_width, 24.0],
                        egui::Label::new(
                            RichText::new(input)
                                .color(ctx.theme.neon_purple)
                                .strong()
                                .size(15.0),
                        ),
                    );

                    if let Some(value) = state.get_text_mut(id) {
                        let height = rows.unwrap_or(1) as f32 * 24.0;
                        let text_edit = egui::TextEdit::multiline(value)
                            .text_color(ctx.theme.base2)
                            .desired_width(ui.available_width())
                            .min_size(Vec2::new(ui.available_width(), height));

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
            let group_frame = egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::same(8))
                .fill(ctx.theme.dark_gray.linear_multiply(0.3))
                .stroke(egui::Stroke::new(
                    1.5,
                    ctx.theme.neon_pink.linear_multiply(0.2),
                ));

            group_frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(group)
                            .color(ctx.theme.neon_pink)
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
