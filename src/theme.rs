use imgui::{Context as ImContext, StyleColor, StyleVar};

#[derive(Debug, Clone)]
pub struct Theme {
    // Colors
    pub neural_blue: [f32; 4],
    pub substrate_dark: [f32; 4],
    pub ghost_white: [f32; 4],
    pub muted_gray: [f32; 4],
    pub tissue_pink: [f32; 4],
    
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
            neural_blue: [0.039, 0.518, 1.0, 1.0],    // #0A84FF
            substrate_dark: [0.11, 0.11, 0.118, 1.0], // #1C1C1E
            ghost_white: [0.949, 0.949, 0.969, 1.0],  // #F2F2F7
            muted_gray: [0.557, 0.557, 0.576, 1.0],   // #8E8E93
            tissue_pink: [1.0, 0.216, 0.373, 1.0],    // #FF375F
            
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
    
    pub fn apply_to_imgui(&self, imgui: &mut ImContext) {
        let style = imgui.style_mut();
        
        // Window styling
        style.window_rounding = self.window_rounding;
        style.window_border_size = self.window_border_size;
        style.window_padding = self.window_padding;
        style.window_title_align = [0.5, 0.5]; // Always center title
        
        // Frame styling
        style.frame_rounding = self.frame_rounding;
        style.frame_border_size = self.frame_border_size;
        style.frame_padding = self.frame_padding;
        
        // Item styling
        style.item_spacing = self.item_spacing;
        style.item_inner_spacing = self.item_inner_spacing;
        
        // Button styling
        style.button_text_align = self.button_text_align;
        
        // Scrollbar styling
        style.scrollbar_size = self.scrollbar_size;
        style.scrollbar_rounding = self.scrollbar_rounding;
        
        // Grab (slider) styling
        style.grab_min_size = self.grab_min_size;
        style.grab_rounding = self.grab_rounding;
        
        // Colors
        style[StyleColor::Text] = self.ghost_white;
        style[StyleColor::TextDisabled] = self.muted_gray;
        
        // Window colors
        style[StyleColor::WindowBg] = self.substrate_dark;
        style[StyleColor::ChildBg] = self.substrate_dark;
        style[StyleColor::PopupBg] = [
            self.substrate_dark[0] + 0.05,
            self.substrate_dark[1] + 0.05,
            self.substrate_dark[2] + 0.05,
            1.0
        ];
        style[StyleColor::Border] = [self.neural_blue[0], self.neural_blue[1], self.neural_blue[2], 0.3];
        style[StyleColor::BorderShadow] = [self.neural_blue[0], self.neural_blue[1], self.neural_blue[2], 0.1];
        
        // Frame colors (inputs, sliders)
        style[StyleColor::FrameBg] = [self.muted_gray[0], self.muted_gray[1], self.muted_gray[2], 0.08];
        style[StyleColor::FrameBgHovered] = [self.neural_blue[0], self.neural_blue[1], self.neural_blue[2], 0.08];
        style[StyleColor::FrameBgActive] = [self.neural_blue[0], self.neural_blue[1], self.neural_blue[2], 0.15];
        
        // Title
        style[StyleColor::TitleBg] = self.substrate_dark;
        style[StyleColor::TitleBgActive] = [
            self.substrate_dark[0] + 0.05,
            self.substrate_dark[1] + 0.05,
            self.substrate_dark[2] + 0.05,
            1.0
        ];
        style[StyleColor::TitleBgCollapsed] = self.substrate_dark;
        
        // Button colors
        style[StyleColor::Button] = self.neural_blue;
        style[StyleColor::ButtonHovered] = [self.neural_blue[0], self.neural_blue[1] * 0.9, self.neural_blue[2], 1.0];
        style[StyleColor::ButtonActive] = [self.neural_blue[0], self.neural_blue[1] * 0.8, self.neural_blue[2], 1.0];
        
        // Check mark
        style[StyleColor::CheckMark] = self.neural_blue;
        
        // Slider
        style[StyleColor::SliderGrab] = self.neural_blue;
        style[StyleColor::SliderGrabActive] = [self.neural_blue[0], self.neural_blue[1] * 0.8, self.neural_blue[2], 1.0];
        
        // Separator
        style[StyleColor::Separator] = [self.muted_gray[0], self.muted_gray[1], self.muted_gray[2], 0.2];
        style[StyleColor::SeparatorHovered] = [self.neural_blue[0], self.neural_blue[1], self.neural_blue[2], 0.3];
        style[StyleColor::SeparatorActive] = self.neural_blue;
        
        // Text selection
        style[StyleColor::TextSelectedBg] = [self.neural_blue[0], self.neural_blue[1], self.neural_blue[2], 0.35];
        
        // Scrollbar - subtle neural theme
        style[StyleColor::ScrollbarBg] = [0.0, 0.0, 0.0, 0.0];  // Transparent background
        style[StyleColor::ScrollbarGrab] = [self.neural_blue[0], self.neural_blue[1], self.neural_blue[2], 0.2];
        style[StyleColor::ScrollbarGrabHovered] = [self.neural_blue[0], self.neural_blue[1], self.neural_blue[2], 0.4];
        style[StyleColor::ScrollbarGrabActive] = [self.neural_blue[0], self.neural_blue[1], self.neural_blue[2], 0.6];
    }
    
    pub fn push_popup_style<'a>(&self, ui: &'a imgui::Ui) -> PopupStyleGuard<'a> {
        PopupStyleGuard {
            _padding: ui.push_style_var(StyleVar::WindowPadding(self.window_padding)),
            _rounding: ui.push_style_var(StyleVar::WindowRounding(self.window_rounding)),
            _border: ui.push_style_var(StyleVar::WindowBorderSize(self.window_border_size)),
            _frame_padding: ui.push_style_var(StyleVar::FramePadding(self.frame_padding)),
            _item_spacing: ui.push_style_var(StyleVar::ItemSpacing(self.item_spacing)),
        }
    }
}

pub struct PopupStyleGuard<'a> {
    _padding: imgui::StyleStackToken<'a>,
    _rounding: imgui::StyleStackToken<'a>,
    _border: imgui::StyleStackToken<'a>,
    _frame_padding: imgui::StyleStackToken<'a>,
    _item_spacing: imgui::StyleStackToken<'a>,
}