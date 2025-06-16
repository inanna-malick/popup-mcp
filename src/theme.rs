use egui::{Color32, Context, Rounding, Stroke};

/// Cyberpunk theme with neon colors and sharp edges
#[derive(Debug, Clone)]
pub struct Theme {
    // Core cyberpunk palette
    pub neon_cyan: Color32,
    pub neon_pink: Color32,
    pub neon_purple: Color32,
    pub electric_blue: Color32,
    pub matrix_green: Color32,
    pub warning_orange: Color32,
    pub deep_black: Color32,
    pub dark_gray: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
}

impl Default for Theme {
    fn default() -> Self {
        Self::cyberpunk()
    }
}

impl Theme {
    pub fn spike_neural() -> Self {
        Self::cyberpunk()
    }
    
    pub fn cyberpunk() -> Self {
        Self {
            neon_cyan: Color32::from_rgb(0, 255, 255),
            neon_pink: Color32::from_rgb(255, 0, 128),
            neon_purple: Color32::from_rgb(191, 0, 255),
            electric_blue: Color32::from_rgb(0, 150, 255),
            matrix_green: Color32::from_rgb(0, 255, 65),
            warning_orange: Color32::from_rgb(255, 140, 0),
            deep_black: Color32::from_rgb(10, 10, 12),
            dark_gray: Color32::from_rgb(25, 25, 30),
            text_primary: Color32::from_rgb(230, 230, 235),
            text_secondary: Color32::from_rgb(160, 160, 170),
        }
    }
    
    pub fn apply_to_egui(&self, ctx: &Context) {
        let mut style = (*ctx.style()).clone();
        let mut visuals = style.visuals.clone();
        
        // Sharp cyberpunk window with neon glow
        visuals.window_rounding = Rounding::ZERO;
        visuals.window_stroke = Stroke::new(2.0, self.neon_cyan);
        visuals.window_shadow.extrusion = 16.0;
        visuals.window_shadow.color = self.neon_cyan.linear_multiply(0.3);
        
        // Tight spacing for compact feel
        style.spacing.button_padding = egui::vec2(10.0, 6.0);
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.menu_margin = egui::Margin::symmetric(10.0, 8.0);
        
        // Dark background with neon text
        visuals.override_text_color = Some(self.text_primary);
        visuals.window_fill = self.deep_black;
        visuals.panel_fill = self.deep_black;
        visuals.faint_bg_color = self.dark_gray;
        
        // Widget states with cyberpunk colors
        visuals.widgets.noninteractive.bg_fill = self.dark_gray;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, self.neon_purple.linear_multiply(0.2));
        visuals.widgets.noninteractive.rounding = Rounding::ZERO;
        
        visuals.widgets.inactive.bg_fill = self.dark_gray;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, self.electric_blue.linear_multiply(0.3));
        visuals.widgets.inactive.rounding = Rounding::ZERO;
        
        visuals.widgets.hovered.bg_fill = self.dark_gray.linear_multiply(1.3);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, self.neon_cyan);
        visuals.widgets.hovered.rounding = Rounding::ZERO;
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.neon_cyan);
        
        visuals.widgets.active.bg_fill = self.neon_pink.linear_multiply(0.15);
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, self.neon_pink);
        visuals.widgets.active.rounding = Rounding::ZERO;
        
        // Neon button highlights
        visuals.widgets.inactive.weak_bg_fill = self.electric_blue.linear_multiply(0.2);
        visuals.widgets.hovered.weak_bg_fill = self.neon_pink.linear_multiply(0.3);
        visuals.widgets.active.weak_bg_fill = self.neon_pink.linear_multiply(0.5);
        
        // Selection and links
        visuals.selection.bg_fill = self.neon_purple.linear_multiply(0.4);
        visuals.hyperlink_color = self.neon_cyan;
        visuals.extreme_bg_color = self.deep_black;
        
        style.visuals = visuals;
        ctx.set_style(style);
    }
}