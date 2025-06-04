use egui::{Color32, Context, Rounding, Stroke};

#[derive(Debug, Clone)]
pub struct Theme {
    // Cyberpunk Colors
    pub neon_cyan: Color32,
    pub neon_pink: Color32,
    pub neon_purple: Color32,
    pub deep_black: Color32,
    pub matrix_green: Color32,
    pub electric_blue: Color32,
    pub warning_orange: Color32,
    pub dark_gray: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    
    // Window
    pub window_rounding: f32,
    pub window_border_size: f32,
    pub window_padding: [f32; 2],
    pub window_glow_strength: f32,
    
    // Frame
    pub frame_rounding: f32,
    pub frame_border_size: f32,
    pub frame_padding: [f32; 2],
    
    // Item
    pub item_spacing: [f32; 2],
    pub item_inner_spacing: [f32; 2],
    
    // Button
    pub button_text_align: [f32; 2],
    
    // Scrollbar
    pub scrollbar_size: f32,
    pub scrollbar_rounding: f32,
    
    // Grab
    pub grab_min_size: f32,
    pub grab_rounding: f32,
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
            // Cyberpunk color palette
            neon_cyan: Color32::from_rgb(0, 255, 255),        // #00FFFF
            neon_pink: Color32::from_rgb(255, 0, 128),        // #FF0080
            neon_purple: Color32::from_rgb(191, 0, 255),      // #BF00FF
            deep_black: Color32::from_rgb(10, 10, 12),        // #0A0A0C
            matrix_green: Color32::from_rgb(0, 255, 65),      // #00FF41
            electric_blue: Color32::from_rgb(0, 150, 255),    // #0096FF
            warning_orange: Color32::from_rgb(255, 140, 0),   // #FF8C00
            dark_gray: Color32::from_rgb(25, 25, 30),         // #19191E
            text_primary: Color32::from_rgb(230, 230, 235),   // #E6E6EB
            text_secondary: Color32::from_rgb(160, 160, 170), // #A0A0AA
            
            // Window styling - Sharp edges with glow
            window_rounding: 0.0,
            window_border_size: 2.0,
            window_padding: [10.0, 8.0],
            window_glow_strength: 0.8,
            
            // Frame styling - Minimal with accent borders
            frame_rounding: 0.0,
            frame_border_size: 1.0,
            frame_padding: [10.0, 6.0],
            
            // Item styling - Tight spacing
            item_spacing: [8.0, 6.0],
            item_inner_spacing: [8.0, 6.0],
            
            // Button styling
            button_text_align: [0.5, 0.5],
            
            // Scrollbar styling
            scrollbar_size: 12.0,
            scrollbar_rounding: 0.0,
            
            // Grab (slider) styling
            grab_min_size: 10.0,
            grab_rounding: 0.0,
        }
    }
    
    pub fn apply_to_egui(&self, ctx: &Context) {
        let mut style = (*ctx.style()).clone();
        let mut visuals = style.visuals.clone();
        
        // Window styling with neon glow
        visuals.window_rounding = Rounding::same(self.window_rounding);
        visuals.window_stroke = Stroke::new(self.window_border_size, self.neon_cyan);
        visuals.window_shadow.extrusion = 16.0;
        visuals.window_shadow.color = self.neon_cyan.linear_multiply(0.3);
        
        // Update spacing
        style.spacing.button_padding = egui::vec2(self.frame_padding[0], self.frame_padding[1]);
        style.spacing.item_spacing = egui::vec2(self.item_spacing[0], self.item_spacing[1]);
        style.spacing.menu_margin = egui::Margin::symmetric(self.window_padding[0], self.window_padding[1]);
        
        // Text colors
        visuals.override_text_color = Some(self.text_primary);
        
        // Window colors - deep black background
        visuals.window_fill = self.deep_black;
        visuals.panel_fill = self.deep_black;
        visuals.faint_bg_color = self.dark_gray;
        
        // Widget colors - cyberpunk style
        visuals.widgets.noninteractive.bg_fill = self.dark_gray;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(self.frame_border_size, self.neon_purple.linear_multiply(0.2));
        visuals.widgets.noninteractive.rounding = Rounding::same(self.frame_rounding);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.text_secondary);
        
        visuals.widgets.inactive.bg_fill = self.dark_gray;
        visuals.widgets.inactive.bg_stroke = Stroke::new(self.frame_border_size, self.electric_blue.linear_multiply(0.3));
        visuals.widgets.inactive.rounding = Rounding::same(self.frame_rounding);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.text_primary);
        
        visuals.widgets.hovered.bg_fill = self.dark_gray.linear_multiply(1.3);
        visuals.widgets.hovered.bg_stroke = Stroke::new(self.frame_border_size, self.neon_cyan);
        visuals.widgets.hovered.rounding = Rounding::same(self.frame_rounding);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.neon_cyan);
        
        visuals.widgets.active.bg_fill = self.neon_pink.linear_multiply(0.15);
        visuals.widgets.active.bg_stroke = Stroke::new(self.frame_border_size, self.neon_pink);
        visuals.widgets.active.rounding = Rounding::same(self.frame_rounding);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, self.text_primary);
        
        // Button specific colors - glowing neon
        visuals.widgets.inactive.weak_bg_fill = self.electric_blue.linear_multiply(0.2);
        visuals.widgets.hovered.weak_bg_fill = self.neon_pink.linear_multiply(0.3);
        visuals.widgets.active.weak_bg_fill = self.neon_pink.linear_multiply(0.5);
        
        // Selection color
        visuals.selection.bg_fill = self.neon_purple.linear_multiply(0.4);
        visuals.selection.stroke = Stroke::new(1.0, self.neon_purple);
        
        // Extreme contrast
        visuals.extreme_bg_color = self.deep_black;
        
        // Hyperlink colors
        visuals.hyperlink_color = self.neon_cyan;
        
        // Code colors
        visuals.code_bg_color = self.dark_gray;
        
        // Warn/Error colors
        visuals.warn_fg_color = self.warning_orange;
        visuals.error_fg_color = self.neon_pink;
        
        style.visuals = visuals;
        ctx.set_style(style);
    }
}