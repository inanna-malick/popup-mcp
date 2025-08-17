use egui::{Color32, Context, Stroke};

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
        Self::soft_focus()  // Default to calming theme
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
    
    pub fn soft_focus() -> Self {
        Self {
            // High contrast, minimal colors for ADHD clarity
            neon_cyan: Color32::from_rgb(59, 130, 246),      // Clear blue for accents
            neon_pink: Color32::from_rgb(239, 68, 68),       // Clear red for alerts
            neon_purple: Color32::from_rgb(139, 92, 246),    // Purple accent
            electric_blue: Color32::from_rgb(59, 130, 246),  // Primary blue
            matrix_green: Color32::from_rgb(34, 197, 94),    // Success green
            warning_orange: Color32::from_rgb(245, 158, 11), // Warning amber
            deep_black: Color32::from_rgb(255, 255, 255),    // Pure white background
            dark_gray: Color32::from_rgb(249, 250, 251),     // Very light gray
            text_primary: Color32::from_rgb(17, 24, 39),     // Near-black text
            text_secondary: Color32::from_rgb(75, 85, 99),   // Dark gray text
        }
    }
    
    pub fn apply_to_egui(&self, ctx: &Context) {
        let mut style = (*ctx.style()).clone();
        let mut visuals = style.visuals.clone();
        
        // Subtle window border (adapts to theme)
        let is_light_theme = self.deep_black.r() > 128;
        let border_width = if is_light_theme { 1.0 } else { 2.0 };
        visuals.window_stroke = Stroke::new(border_width, self.electric_blue.linear_multiply(0.3));
        visuals.window_shadow.color = Color32::from_black_alpha(20);
        
        // Compact spacing for efficiency
        style.spacing.button_padding = egui::vec2(8.0, 4.0);
        style.spacing.item_spacing = egui::vec2(6.0, 4.0);
        
        // Background and text colors
        visuals.override_text_color = Some(self.text_primary);
        visuals.window_fill = self.deep_black;
        visuals.panel_fill = self.deep_black;
        visuals.faint_bg_color = self.dark_gray;
        
        // High contrast widget states for visibility
        let border_color = if is_light_theme {
            Color32::from_gray(180)  // Gray borders on light background
        } else {
            Color32::from_gray(100)  // Lighter borders on dark background
        };
        
        visuals.widgets.noninteractive.bg_fill = self.dark_gray;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, border_color);
        
        visuals.widgets.inactive.bg_fill = self.dark_gray;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, border_color);
        
        visuals.widgets.hovered.bg_fill = self.electric_blue.linear_multiply(0.1);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.5, self.electric_blue);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.text_primary);
        
        visuals.widgets.active.bg_fill = self.neon_cyan;  // Cyan background for selection
        visuals.widgets.active.bg_stroke = Stroke::new(2.0, Color32::BLACK);  // Black outline
        
        // Button styling for visibility
        visuals.widgets.inactive.weak_bg_fill = if is_light_theme {
            Color32::from_gray(240)
        } else {
            self.electric_blue.linear_multiply(0.2)
        };
        visuals.widgets.hovered.weak_bg_fill = self.electric_blue.linear_multiply(0.1);
        visuals.widgets.active.weak_bg_fill = self.electric_blue.linear_multiply(0.2);
        
        // Selection and links with high contrast
        visuals.selection.bg_fill = self.electric_blue.linear_multiply(0.2);
        visuals.hyperlink_color = self.electric_blue;
        visuals.extreme_bg_color = self.deep_black;
        
        style.visuals = visuals;
        ctx.set_style(style);
    }
}