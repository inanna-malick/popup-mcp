use imgui::{Ui, StyleColor};
use crate::models::{Element, PopupState};
use crate::theme::Theme;

pub trait WidgetRenderer {
    fn render(&self, ui: &Ui, state: &mut PopupState, theme: &Theme) -> bool;
}

pub fn render_text(ui: &Ui, text: &str, is_header: bool, theme: &Theme) {
    if is_header && text.to_uppercase() == text {
        // This looks like a header - style it specially
        let _color = ui.push_style_color(StyleColor::Text, theme.neural_blue);
        ui.text_wrapped(text);
        ui.separator();
    } else {
        ui.text_wrapped(text);
    }
}

pub fn render_slider(ui: &Ui, label: &str, state: &mut PopupState) {
    if let Some(value) = state.sliders.get_mut(label) {
        ui.slider(label, *value as f32, *value as f32, value);
    }
}

pub fn render_checkbox(ui: &Ui, label: &str, state: &mut PopupState) {
    if let Some(value) = state.checkboxes.get_mut(label) {
        ui.checkbox(label, value);
    }
}

pub fn render_textbox(ui: &Ui, label: &str, placeholder: &Option<String>, rows: &Option<u32>, state: &mut PopupState) {
    ui.text(label);
    if let Some(value) = state.textboxes.get_mut(label) {
        if let Some(rows) = rows {
            if *rows > 1 {
                ui.input_text_multiline(
                    &format!("##{}", label),
                    value,
                    [400.0, (*rows as f32) * 20.0],
                )
                .build();
            } else {
                let mut input = ui.input_text(format!("##{}", label), value);
                if let Some(hint) = placeholder {
                    input = input.hint(hint);
                }
                input.build();
            }
        } else {
            let mut input = ui.input_text(format!("##{}", label), value);
            if let Some(hint) = placeholder {
                input = input.hint(hint);
            }
            input.build();
        }
    }
}

pub fn render_choice(ui: &Ui, label: &str, options: &[String], state: &mut PopupState) {
    ui.text(label);
    if let Some(selected) = state.choices.get_mut(label) {
        for (i, option) in options.iter().enumerate() {
            ui.radio_button(option, selected, i);
        }
    }
}

pub fn render_multiselect(ui: &Ui, label: &str, options: &[String], state: &mut PopupState) {
    ui.text(label);
    if let Some(selections) = state.multiselects.get_mut(label) {
        for (i, option) in options.iter().enumerate() {
            if i < selections.len() {
                ui.checkbox(option, &mut selections[i]);
            }
        }
    }
}

pub fn render_buttons(ui: &Ui, buttons: &[String], state: &mut PopupState, theme: &Theme) -> bool {
    ui.separator();
    ui.spacing(); // Just one spacing before buttons
    
    let button_width = 90.0; // Slightly wider for larger text
    let button_height = 26.0; // Increased height for larger text
    let total_width = buttons.len() as f32 * (button_width + 8.0);
    let start_x = (ui.window_size()[0] - total_width) / 2.0;
    
    ui.set_cursor_pos([start_x, ui.cursor_pos()[1]]);
    
    // Style buttons with neural blue
    let _button_color = ui.push_style_color(StyleColor::Button, theme.neural_blue);
    let _button_hover = ui.push_style_color(StyleColor::ButtonHovered, [theme.neural_blue[0], theme.neural_blue[1] * 0.9, theme.neural_blue[2], 1.0]);
    let _button_active = ui.push_style_color(StyleColor::ButtonActive, [theme.neural_blue[0], theme.neural_blue[1] * 0.8, theme.neural_blue[2], 1.0]);
    
    for (i, button) in buttons.iter().enumerate() {
        if ui.button_with_size(button, [button_width, button_height]) {
            state.button_clicked = Some(button.clone());
            return false;
        }
        if i < buttons.len() - 1 {
            ui.same_line();
        }
    }
    
    true
}