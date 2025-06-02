use anyhow::Result;
use glow::HasContext;
use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextAttributesBuilder, PossiblyCurrentContext};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{Surface, SurfaceAttributesBuilder, SwapInterval, WindowSurface};
use glutin_winit::DisplayBuilder;
use imgui::{Context as ImContext, FontConfig, FontSource, Textures};
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
use std::sync::mpsc;
use std::thread;

pub fn render_popup(definition: PopupDefinition) -> Result<PopupResult> {
    // Use a channel to get the result from the GUI thread
    let (tx, rx) = mpsc::channel();
    
    // Spawn a new thread for each popup
    thread::spawn(move || {
        let result = render_popup_thread(definition);
        tx.send(result).ok();
    });
    
    // Wait for the result
    rx.recv()
        .map_err(|_| anyhow::anyhow!("Failed to receive popup result"))?
}

fn render_popup_thread(definition: PopupDefinition) -> Result<PopupResult> {

    // Create event loop and window
    let event_loop = EventLoop::new()?;
    
    let window_builder = WindowBuilder::new()
        .with_title(&definition.title)
        .with_inner_size(LogicalSize::new(800.0, 600.0))
        .with_visible(false);

    // Create OpenGL context
    let (window, gl_context, gl_surface, gl) = create_gl_context(window_builder, &event_loop)?;
    
    // Create imgui context
    let mut imgui = ImContext::create();
    imgui.set_ini_filename(None);
    
    // Configure fonts
    let hidpi_factor = window.scale_factor();
    let font_size = (13.0 * hidpi_factor) as f32;
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
    
    event_loop.run( |event, window_target| {
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
                    gl.clear_color(0.1, 0.1, 0.1, 1.0);
                    gl.clear(glow::COLOR_BUFFER_BIT);
                }
                
                let ui = imgui.frame();
                
                // Center the popup
                let window_size = ui.io().display_size;
                let center = [window_size[0] / 2.0, window_size[1] / 2.0];
                
                // Create window
                let token = ui.push_style_var(imgui::StyleVar::WindowRounding(5.0));
                if show_popup {
                    ui.window(&title)
                        .position(center, imgui::Condition::FirstUseEver)
                        .position_pivot([0.5, 0.5])
                        .size([500.0, 400.0], imgui::Condition::FirstUseEver)
                        .resizable(false)
                        .collapsible(false)
                        .menu_bar(false)
                        .build(|| {
                            show_popup = render_elements(&ui, &elements, &mut state);
                        });
                }
                token.pop();
                
                // If popup was closed, exit
                if !show_popup || state.button_clicked.is_some() {
                    result = Some(PopupResult::from_state(&state));
                    window_target.exit();
                }
                
                platform.prepare_render(&ui, &window);
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
    for element in elements {
        match element {
            Element::Text(text) => {
                ui.text(text);
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
                
                let button_width = 120.0;
                let total_width = buttons.len() as f32 * (button_width + 10.0);
                let start_x = (ui.window_size()[0] - total_width) / 2.0;
                
                ui.set_cursor_pos([start_x, ui.cursor_pos()[1]]);
                
                for (i, button) in buttons.iter().enumerate() {
                    if ui.button_with_size(button, [button_width, 30.0]) {
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
