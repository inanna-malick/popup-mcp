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
    pub base2: Color32,
    pub base3: Color32,
}

impl Default for Theme {
    fn default() -> Self {
        Self::solarized_dark() // Default to Solarized Dark
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
            base2: Color32::WHITE,
            base3: Color32::WHITE,
        }
    }

    pub fn soft_focus() -> Self {
        Self {
            // High contrast, minimal colors for ADHD clarity
            neon_cyan: Color32::from_rgb(59, 130, 246), // Clear blue for accents
            neon_pink: Color32::from_rgb(239, 68, 68),  // Clear red for alerts
            neon_purple: Color32::from_rgb(139, 92, 246), // Purple accent
            electric_blue: Color32::from_rgb(59, 130, 246), // Primary blue
            matrix_green: Color32::from_rgb(34, 197, 94), // Success green
            warning_orange: Color32::from_rgb(245, 158, 11), // Warning amber
            deep_black: Color32::from_rgb(255, 255, 255), // Pure white background
            dark_gray: Color32::from_rgb(249, 250, 251), // Very light gray
            text_primary: Color32::from_rgb(17, 24, 39), // Near-black text
            text_secondary: Color32::from_rgb(75, 85, 99), // Dark gray text
            base2: Color32::BLACK,
            base3: Color32::BLACK,
        }
    }

    pub fn solarized_dark() -> Self {
        Self {
            // Solarized Dark color palette
            neon_cyan: Color32::from_rgb(42, 161, 152), // cyan
            neon_pink: Color32::from_rgb(211, 54, 130), // magenta
            neon_purple: Color32::from_rgb(108, 113, 196), // violet
            electric_blue: Color32::from_rgb(38, 139, 210), // blue
            matrix_green: Color32::from_rgb(133, 153, 0), // green
            warning_orange: Color32::from_rgb(203, 75, 22), // orange/red
            deep_black: Color32::from_rgb(0, 43, 54),   // base03 (darkest background)
            dark_gray: Color32::from_rgb(7, 54, 66),    // base02 (background highlights)
            text_primary: Color32::from_rgb(147, 161, 161), // base1 (primary content)
            text_secondary: Color32::from_rgb(101, 123, 131), // base0 (secondary content)
            base2: Color32::from_rgb(238, 232, 213),    // base2
            base3: Color32::from_rgb(253, 246, 227),    // base3 (brightest content)
        }
    }

    pub fn solarized_light() -> Self {
        Self {
            // Solarized Light color palette
            neon_cyan: Color32::from_rgb(42, 161, 152), // cyan
            neon_pink: Color32::from_rgb(211, 54, 130), // magenta
            neon_purple: Color32::from_rgb(108, 113, 196), // violet
            electric_blue: Color32::from_rgb(38, 139, 210), // blue
            matrix_green: Color32::from_rgb(133, 153, 0), // green
            warning_orange: Color32::from_rgb(203, 75, 22), // orange/red
            deep_black: Color32::from_rgb(253, 246, 227), // base3 (lightest background)
            dark_gray: Color32::from_rgb(238, 232, 213), // base2 (background highlights)
            text_primary: Color32::from_rgb(88, 110, 117), // base01 (primary content)
            text_secondary: Color32::from_rgb(131, 148, 150), // base00 (secondary content)
            base2: Color32::from_rgb(7, 54, 66),        // base02
            base3: Color32::from_rgb(0, 43, 54),        // base03 (darkest content)
        }
    }

    pub fn apply_to_egui(&self, ctx: &Context) {
        let mut style = (*ctx.style()).clone();
        let mut visuals = style.visuals.clone();

        // Standard Solarized border (no opacity hacks)
        let is_light_theme = self.deep_black.r() > 128;
        let border_width = if is_light_theme { 1.0 } else { 1.0 };
        visuals.window_stroke = Stroke::new(border_width, self.text_secondary); // base0/base00
        visuals.window_shadow.color = Color32::from_black_alpha(25);

        // Increased padding and spacing
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.window_margin = egui::Margin::same(10);

        // Background and text colors
        visuals.override_text_color = Some(self.text_primary);
        visuals.window_fill = self.deep_black;
        visuals.panel_fill = self.deep_black;
        visuals.faint_bg_color = self.dark_gray;

        // Use base01/base02 for all structural lines
        let border_color = if is_light_theme {
            Color32::from_rgb(147, 161, 161) // base1
        } else {
            Color32::from_rgb(88, 110, 117) // base01
        };

        visuals.widgets.noninteractive.bg_fill = self.dark_gray;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, border_color);

        visuals.widgets.inactive.bg_fill = self.dark_gray;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, border_color);

        visuals.widgets.hovered.bg_fill = self.dark_gray;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, self.electric_blue);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.base3);

        visuals.widgets.active.bg_fill = self.dark_gray;
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, self.neon_cyan);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, self.base3);

        // Button styling - solid colors
        visuals.widgets.inactive.weak_bg_fill = self.dark_gray;
        visuals.widgets.hovered.weak_bg_fill = if is_light_theme { self.base2 } else { self.text_secondary };
        visuals.widgets.active.weak_bg_fill = self.electric_blue;

        // "Sunken" widgets (inputs, combo boxes)
        visuals.extreme_bg_color = self.dark_gray;

        // Selection and links
        visuals.selection.bg_fill = self.electric_blue;
        visuals.selection.stroke = Stroke::new(1.0, self.base3);
        visuals.hyperlink_color = self.neon_cyan;

        style.visuals = visuals;
        ctx.set_style(style);
    }
}
