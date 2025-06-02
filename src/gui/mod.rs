use anyhow::Result;
use glow::HasContext;
use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextAttributesBuilder, PossiblyCurrentContext};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{Surface, SurfaceAttributesBuilder, SwapInterval, WindowSurface};
use glutin_winit::DisplayBuilder;
use imgui::{Context as ImContext, FontConfig, FontSource, Textures, StyleColor};
use imgui_glow_renderer::glow::Context as GlowContext;
use imgui_glow_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use raw_window_handle::HasRawWindowHandle;
use std::ffi::CString;
use std::num::NonZeroU32;
use std::time::Instant;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use crate::models::{Element, PopupDefinition, PopupResult, PopupState};

// Spike color palette
const NEURAL_BLUE: [f32; 4] = [0.039, 0.518, 1.0, 1.0];    // #0A84FF
const SUBSTRATE_DARK: [f32; 4] = [0.11, 0.11, 0.118, 1.0]; // #1C1C1E
const GHOST_WHITE: [f32; 4] = [0.949, 0.949, 0.969, 1.0];  // #F2F2F7
const MUTED_GRAY: [f32; 4] = [0.557, 0.557, 0.576, 1.0];   // #8E8E93
const TISSUE_PINK: [f32; 4] = [1.0, 0.216, 0.373, 1.0];    // #FF375F

pub fn render_popup(definition: PopupDefinition) -> Result<PopupResult> {
    // Calculate approximate window size based on content
    let (width, height) = calculate_window_size(&definition);

    // Create event loop and window
    let event_loop = EventLoop::new()?;
    
    let window_builder = WindowBuilder::new()
        .with_title(&definition.title)
        .with_inner_size(LogicalSize::new(width, height))
        .with_resizable(false) // Prevent user resizing
        .with_visible(false);

    // Create OpenGL context
    let (window, gl_context, gl_surface, gl) = create_gl_context(window_builder, &event_loop)?;
    
    // Create imgui context
    let mut imgui = ImContext::create();
    imgui.set_ini_filename(None);
    
    // Apply Spike neural interface theme
    apply_spike_theme(&mut imgui);
    
    // Configure fonts
    let hidpi_factor = window.scale_factor();
    let font_size = (14.0 * hidpi_factor) as f32; // Slightly larger for readability
    
    // Try to load monospace font first
    imgui.fonts().add_font(&[FontSource::DefaultFontData {
        config: Some(FontConfig {
            size_pixels: font_size,
            ..FontConfig::default()
        }),
    }]);
    
    // Initialize platform and renderer
    let mut platform = WinitPlatform::init(&mut imgui);
    platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Rounded);
    
    let mut textures = Textures::<glow::Texture>::new();
    let mut renderer = Renderer::initialize(&gl, &mut imgui, &mut textures, true)?;
    
    window.set_visible(true);
    
    // Popup state
    let mut state = PopupState::new(&definition);
    let mut show_popup = true;
    let mut last_frame = Instant::now();
    
    let mut result = None;
    
    // Main event loop
    let title = definition.title.clone();
    let elements = definition.elements.clone();
    
    event_loop.run(|event, window_target| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                state.button_clicked = Some("cancel".to_string());
                result = Some(PopupResult::from_state(&state));
                window_target.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                if new_size.width > 0 && new_size.height > 0 {
                    gl_surface.resize(
                        &gl_context,
                        NonZeroU32::new(new_size.width).unwrap(),
                        NonZeroU32::new(new_size.height).unwrap(),
                    );
                }
            }
            Event::AboutToWait => {
                let now = Instant::now();
                imgui.io_mut().update_delta_time(now - last_frame);
                last_frame = now;
                platform.prepare_frame(imgui.io_mut(), &window).unwrap();
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                unsafe {
                    // Spike substrate dark background
                    gl.clear_color(SUBSTRATE_DARK[0], SUBSTRATE_DARK[1], SUBSTRATE_DARK[2], 1.0);
                    gl.clear(glow::COLOR_BUFFER_BIT);
                }
                
                // Build UI in a scope to drop the borrow before rendering
                {
                    let ui = imgui.frame();
                    
                    // Fill the entire window with the popup
                    let window_size = ui.io().display_size;
                    
                    // Style adjustments for Spike theme
                    let _padding = ui.push_style_var(imgui::StyleVar::WindowPadding([24.0, 20.0]));
                    let _rounding = ui.push_style_var(imgui::StyleVar::WindowRounding(12.0));
                    let _border = ui.push_style_var(imgui::StyleVar::WindowBorderSize(1.0));
                    let _frame_padding = ui.push_style_var(imgui::StyleVar::FramePadding([12.0, 8.0]));
                    let _item_spacing = ui.push_style_var(imgui::StyleVar::ItemSpacing([8.0, 12.0]));
                    
                    if show_popup {
                        ui.window(&title)
                            .position([0.0, 0.0], imgui::Condition::Always)
                            .size(window_size, imgui::Condition::Always)
                            .resizable(false)
                            .movable(false)
                            .collapsible(false)
                            .title_bar(true)
                            .build(|| {
                                show_popup = render_elements(&ui, &elements, &mut state);
                            });
                    }
                    
                    // If popup was closed, exit
                    if !show_popup || state.button_clicked.is_some() {
                        result = Some(PopupResult::from_state(&state));
                        window_target.exit();
                    }
                    
                    platform.prepare_render(&ui, &window);
                }
                
                let draw_data = imgui.render();
                
                if draw_data.draw_lists().count() > 0 {
                    renderer.render(&gl, &textures, draw_data).unwrap();
                }
                
                gl_surface.swap_buffers(&gl_context).unwrap();
            }
            _ => {
                platform.handle_event(imgui.io_mut(), &window, &event);
            }
        }
    })?;
    
    result.ok_or_else(|| anyhow::anyhow!("Popup closed without result"))
}

fn render_elements(ui: &imgui::Ui, elements: &[Element], state: &mut PopupState) -> bool {
    // First, check if there's a title-like text at the beginning
    let mut element_iter = elements.iter().peekable();
    let mut is_first = true;
    
    while let Some(element) = element_iter.next() {
        match element {
            Element::Text(text) => {
                if is_first && text.to_uppercase() == *text {
                    // This looks like a header - style it specially
                    let _color = ui.push_style_color(StyleColor::Text, NEURAL_BLUE);
                    ui.text(text);
                    ui.separator();
                    ui.spacing();
                } else {
                    ui.text(text);
                }
                is_first = false;
            }
            
            Element::Slider { label, min, max, .. } => {
                if let Some(value) = state.sliders.get_mut(label) {
                    ui.slider(label, *min, *max, value);
                }
            }
            
            Element::Checkbox { label, .. } => {
                if let Some(value) = state.checkboxes.get_mut(label) {
                    ui.checkbox(label, value);
                }
            }
            
            Element::Textbox { label, placeholder, rows } => {
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
            
            Element::Choice { label, options } => {
                ui.text(label);
                if let Some(selected) = state.choices.get_mut(label) {
                    for (i, option) in options.iter().enumerate() {
                        ui.radio_button(option, selected, i);
                    }
                }
            }
            
            Element::Group { label, elements } => {
                ui.text(label);
                ui.indent();
                let keep_open = render_elements(ui, elements, state);
                ui.unindent();
                if !keep_open {
                    return false;
                }
            }
            
            Element::Buttons(buttons) => {
                ui.separator();
                ui.spacing();
                ui.spacing(); // Extra spacing before buttons
                
                let button_width = 140.0; // Wider buttons
                let button_height = 36.0; // Taller buttons
                let total_width = buttons.len() as f32 * (button_width + 10.0);
                let start_x = (ui.window_size()[0] - total_width) / 2.0;
                
                ui.set_cursor_pos([start_x, ui.cursor_pos()[1]]);
                
                // Style buttons with neural blue
                let _button_color = ui.push_style_color(StyleColor::Button, NEURAL_BLUE);
                let _button_hover = ui.push_style_color(StyleColor::ButtonHovered, [NEURAL_BLUE[0], NEURAL_BLUE[1] * 0.9, NEURAL_BLUE[2], 1.0]);
                let _button_active = ui.push_style_color(StyleColor::ButtonActive, [NEURAL_BLUE[0], NEURAL_BLUE[1] * 0.8, NEURAL_BLUE[2], 1.0]);
                
                for (i, button) in buttons.iter().enumerate() {
                    if ui.button_with_size(button, [button_width, button_height]) {
                        state.button_clicked = Some(button.clone());
                        return false;
                    }
                    if i < buttons.len() - 1 {
                        ui.same_line();
                    }
                }
            }
        }
        
        ui.spacing();
    }
    
    true
}

fn create_gl_context(
    window_builder: WindowBuilder,
    event_loop: &EventLoop<()>,
) -> Result<(
    winit::window::Window,
    PossiblyCurrentContext,
    Surface<WindowSurface>,
    GlowContext,
)> {
    let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

    let (window, gl_config) = display_builder
        .build(event_loop, ConfigTemplateBuilder::new(), |configs| {
            configs
                .reduce(|accum, config| {
                    let config_transparency = config.supports_transparency().unwrap_or(false);
                    let accum_transparency = accum.supports_transparency().unwrap_or(false);
                    
                    match (config_transparency, accum_transparency) {
                        (true, false) => config,
                        (false, true) => accum,
                        _ => {
                            if config.num_samples() > accum.num_samples() {
                                config
                            } else {
                                accum
                            }
                        }
                    }
                })
                .unwrap()
        })
        .map_err(|e| anyhow::anyhow!("Failed to build display: {:?}", e))?;

    let window = window.unwrap();
    let gl_display = gl_config.display();

    let context_attributes =
        ContextAttributesBuilder::new().build(Some(window.raw_window_handle()));

    let gl_context = unsafe { gl_display.create_context(&gl_config, &context_attributes)? };

    let surface_attributes = SurfaceAttributesBuilder::<WindowSurface>::new().build(
        window.raw_window_handle(),
        NonZeroU32::new(800).unwrap(),
        NonZeroU32::new(600).unwrap(),
    );

    let gl_surface = unsafe { gl_display.create_window_surface(&gl_config, &surface_attributes)? };
    let gl_context = gl_context.make_current(&gl_surface)?;
    
    gl_surface.set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))?;

    let gl = unsafe {
        GlowContext::from_loader_function(|s| {
            let c_str = CString::new(s).unwrap();
            gl_display.get_proc_address(&c_str) as *const _
        })
    };

    Ok((window, gl_context, gl_surface, gl))
}

fn calculate_window_size(definition: &PopupDefinition) -> (f32, f32) {
    let mut height: f32 = 60.0; // Title bar + window chrome
    let mut max_width: f32 = 350.0; // Minimum width
    
    // Account for special header styling
    let mut has_header = false;
    
    // Estimate size for each element
    for (i, element) in definition.elements.iter().enumerate() {
        match element {
            Element::Text(text) => {
                if i == 0 && text.to_uppercase() == *text {
                    // Header text gets extra space for separator
                    height += 30.0; // Text + separator + extra spacing
                    has_header = true;
                } else {
                    height += 25.0;
                }
                max_width = max_width.max(text.len() as f32 * 7.0 + 40.0);
            }
            Element::Slider { label, .. } => {
                height += 45.0; // Label + slider + spacing
                max_width = max_width.max(label.len() as f32 * 7.0 + 220.0);
            }
            Element::Checkbox { label, .. } => {
                height += 30.0;
                max_width = max_width.max(label.len() as f32 * 7.0 + 80.0);
            }
            Element::Textbox { label, rows, .. } => {
                height += 25.0; // Label
                height += 40.0 * (rows.unwrap_or(1) as f32); // Input field with padding
                max_width = max_width.max(400.0);
            }
            Element::Choice { label, options } => {
                height += 25.0; // Label
                height += 30.0 * options.len() as f32; // Each radio option
                let longest_option = options.iter().map(|s| s.len()).max().unwrap_or(0);
                max_width = max_width.max((longest_option as f32) * 8.0 + 100.0);
            }
            Element::Group { elements, .. } => {
                height += 25.0; // Group label
                for _sub_element in elements {
                    height += 30.0;
                }
            }
            Element::Buttons(buttons) => {
                height += 20.0; // Extra spacing before buttons
                height += 15.0; // Separator
                height += 40.0; // Button height with padding
                height += 25.0; // Bottom padding
                let button_width = buttons.len() as f32 * 160.0;
                max_width = max_width.max(button_width);
            }
        }
        height += 12.0; // Item spacing between elements
    }
    
    // Add window padding (top and bottom) - more generous
    height += 50.0; // Extra padding for window chrome and safety margin
    
    max_width = max_width.min(600.0); // Cap maximum width
    
    (max_width, height)
}

fn apply_spike_theme(imgui: &mut ImContext) {
    let style = imgui.style_mut();
    
    // Window styling
    style.window_rounding = 12.0;
    style.window_border_size = 1.0;
    style.window_padding = [24.0, 20.0];
    style.window_title_align = [0.5, 0.5]; // Center title
    
    // Frame styling
    style.frame_rounding = 8.0;
    style.frame_border_size = 1.0;
    style.frame_padding = [12.0, 6.0];
    
    // Item styling
    style.item_spacing = [8.0, 12.0];
    style.item_inner_spacing = [8.0, 6.0];
    
    // Button styling
    style.button_text_align = [0.5, 0.5];
    
    // Scrollbar styling
    style.scrollbar_size = 14.0;
    style.scrollbar_rounding = 9.0;
    
    // Grab (slider) styling
    style.grab_min_size = 12.0;
    style.grab_rounding = 12.0;
    
    // Colors
    style[StyleColor::Text] = GHOST_WHITE;
    style[StyleColor::TextDisabled] = MUTED_GRAY;
    
    // Window colors
    style[StyleColor::WindowBg] = SUBSTRATE_DARK;
    style[StyleColor::ChildBg] = SUBSTRATE_DARK;
    style[StyleColor::PopupBg] = [SUBSTRATE_DARK[0] + 0.05, SUBSTRATE_DARK[1] + 0.05, SUBSTRATE_DARK[2] + 0.05, 1.0];
    style[StyleColor::Border] = [NEURAL_BLUE[0], NEURAL_BLUE[1], NEURAL_BLUE[2], 0.3];
    style[StyleColor::BorderShadow] = [NEURAL_BLUE[0], NEURAL_BLUE[1], NEURAL_BLUE[2], 0.1];
    
    // Frame colors (inputs, sliders)
    style[StyleColor::FrameBg] = [MUTED_GRAY[0], MUTED_GRAY[1], MUTED_GRAY[2], 0.08];
    style[StyleColor::FrameBgHovered] = [NEURAL_BLUE[0], NEURAL_BLUE[1], NEURAL_BLUE[2], 0.08];
    style[StyleColor::FrameBgActive] = [NEURAL_BLUE[0], NEURAL_BLUE[1], NEURAL_BLUE[2], 0.15];
    
    // Title
    style[StyleColor::TitleBg] = SUBSTRATE_DARK;
    style[StyleColor::TitleBgActive] = [SUBSTRATE_DARK[0] + 0.05, SUBSTRATE_DARK[1] + 0.05, SUBSTRATE_DARK[2] + 0.05, 1.0];
    style[StyleColor::TitleBgCollapsed] = SUBSTRATE_DARK;
    
    // Button colors
    style[StyleColor::Button] = NEURAL_BLUE;
    style[StyleColor::ButtonHovered] = [NEURAL_BLUE[0], NEURAL_BLUE[1] * 0.9, NEURAL_BLUE[2], 1.0];
    style[StyleColor::ButtonActive] = [NEURAL_BLUE[0], NEURAL_BLUE[1] * 0.8, NEURAL_BLUE[2], 1.0];
    
    // Check mark
    style[StyleColor::CheckMark] = NEURAL_BLUE;
    
    // Slider
    style[StyleColor::SliderGrab] = NEURAL_BLUE;
    style[StyleColor::SliderGrabActive] = [NEURAL_BLUE[0], NEURAL_BLUE[1] * 0.8, NEURAL_BLUE[2], 1.0];
    
    // Separator
    style[StyleColor::Separator] = [MUTED_GRAY[0], MUTED_GRAY[1], MUTED_GRAY[2], 0.2];
    style[StyleColor::SeparatorHovered] = [NEURAL_BLUE[0], NEURAL_BLUE[1], NEURAL_BLUE[2], 0.3];
    style[StyleColor::SeparatorActive] = NEURAL_BLUE;
    
    // Text selection
    style[StyleColor::TextSelectedBg] = [NEURAL_BLUE[0], NEURAL_BLUE[1], NEURAL_BLUE[2], 0.35];
    
    // Scrollbar
    style[StyleColor::ScrollbarBg] = [SUBSTRATE_DARK[0], SUBSTRATE_DARK[1], SUBSTRATE_DARK[2], 0.0];
    style[StyleColor::ScrollbarGrab] = [MUTED_GRAY[0], MUTED_GRAY[1], MUTED_GRAY[2], 0.3];
    style[StyleColor::ScrollbarGrabHovered] = [MUTED_GRAY[0], MUTED_GRAY[1], MUTED_GRAY[2], 0.5];
    style[StyleColor::ScrollbarGrabActive] = [NEURAL_BLUE[0], NEURAL_BLUE[1], NEURAL_BLUE[2], 0.6];
}
