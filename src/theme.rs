use egui::{Color32, Context, Rounding, Stroke};

#[derive(Debug, Clone)]
pub struct Theme {
    // Colors
    pub neural_blue: Color32,
    pub substrate_dark: Color32,
    pub ghost_white: Color32,
    pub muted_gray: Color32,
    pub tissue_pink: Color32,
    
    // Window
    pub window_rounding: f32,
    pub window_border_size: f32,
    pub window_padding: [f32; 2],
    
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
        Self::spike_neural()
    }
}

impl Theme {
    pub fn spike_neural() -> Self {
        Self {
            // Spike color palette
            neural_blue: Color32::from_rgb(10, 132, 255),     // #0A84FF
            substrate_dark: Color32::from_rgb(28, 28, 30),    // #1C1C1E
            ghost_white: Color32::from_rgb(242, 242, 247),    // #F2F2F7
            muted_gray: Color32::from_rgb(142, 142, 147),     // #8E8E93
            tissue_pink: Color32::from_rgb(255, 55, 95),      // #FF375F
            
            // Window styling - ULTRA DENSE
            window_rounding: 4.0,
            window_border_size: 1.0,
            window_padding: [8.0, 6.0],
            
            // Frame styling - Dense but readable
            frame_rounding: 3.0,
            frame_border_size: 1.0,
            frame_padding: [8.0, 4.0],
            
            // Item styling - Dense but readable
            item_spacing: [6.0, 4.0],
            item_inner_spacing: [6.0, 4.0],
            
            // Button styling
            button_text_align: [0.5, 0.5],
            
            // Scrollbar styling
            scrollbar_size: 10.0,
            scrollbar_rounding: 5.0,
            
            // Grab (slider) styling
            grab_min_size: 8.0,
            grab_rounding: 8.0,
        }
    }
    
    pub fn apply_to_egui(&self, ctx: &Context) {
        let mut style = (*ctx.style()).clone();
        let mut visuals = style.visuals.clone();
        
        // Window styling
        visuals.window_rounding = Rounding::same(self.window_rounding);
        visuals.window_stroke = Stroke::new(self.window_border_size, self.neural_blue.linear_multiply(0.3));
        
        // Update spacing
        style.spacing.button_padding = egui::vec2(self.frame_padding[0], self.frame_padding[1]);
        style.spacing.item_spacing = egui::vec2(self.item_spacing[0], self.item_spacing[1]);
        style.spacing.menu_margin = egui::Margin::symmetric(self.window_padding[0], self.window_padding[1]);
        
        // Colors
        visuals.override_text_color = Some(self.ghost_white);
        
        // Window colors
        visuals.window_fill = self.substrate_dark;
        visuals.panel_fill = self.substrate_dark;
        visuals.faint_bg_color = self.substrate_dark;
        
        // Widget colors
        visuals.widgets.noninteractive.bg_fill = self.muted_gray.linear_multiply(0.08);
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(self.frame_border_size, self.muted_gray.linear_multiply(0.2));
        visuals.widgets.noninteractive.rounding = Rounding::same(self.frame_rounding);
        
        visuals.widgets.inactive.bg_fill = self.muted_gray.linear_multiply(0.08);
        visuals.widgets.inactive.bg_stroke = Stroke::new(self.frame_border_size, self.muted_gray.linear_multiply(0.2));
        visuals.widgets.inactive.rounding = Rounding::same(self.frame_rounding);
        
        visuals.widgets.hovered.bg_fill = self.neural_blue.linear_multiply(0.08);
        visuals.widgets.hovered.bg_stroke = Stroke::new(self.frame_border_size, self.neural_blue.linear_multiply(0.3));
        visuals.widgets.hovered.rounding = Rounding::same(self.frame_rounding);
        
        visuals.widgets.active.bg_fill = self.neural_blue.linear_multiply(0.15);
        visuals.widgets.active.bg_stroke = Stroke::new(self.frame_border_size, self.neural_blue);
        visuals.widgets.active.rounding = Rounding::same(self.frame_rounding);
        
        // Button specific colors
        visuals.widgets.inactive.weak_bg_fill = self.neural_blue;
        visuals.widgets.hovered.weak_bg_fill = self.neural_blue.linear_multiply(0.9);
        visuals.widgets.active.weak_bg_fill = self.neural_blue.linear_multiply(0.8);
        
        // Selection color
        visuals.selection.bg_fill = self.neural_blue.linear_multiply(0.35);
        
        style.visuals = visuals;
        ctx.set_style(style);
    }
}