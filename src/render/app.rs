use iced_wgpu::{graphics::Viewport, wgpu, Engine, Renderer};
use iced_winit::{
    clipboard::Clipboard,
    conversion::{cursor_position, mouse_interaction, window_event},
    core::{mouse, renderer, Font, Pixels, Size, Theme},
    runtime::{program, Debug},
    winit::{
        application::ApplicationHandler,
        dpi::PhysicalPosition,
        event::WindowEvent,
        event_loop::ActiveEventLoop,
        event_loop::ControlFlow,
        keyboard::ModifiersState,
        window::{Window, WindowId},
    },
};

use crate::render::{
    controls::Controls,
    message::{ConwayMessage, PolybladeMessage, PresetMessage},
    pipeline::{FragUniforms, ModelUniforms, PolyhedronPrimitive, Scene, Texture},
};

#[cfg(target_arch = "wasm32")]
pub use iced::time::Instant;
#[cfg(not(target_arch = "wasm32"))]
pub use std::time::Instant;

pub struct Graphics<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    viewport: Viewport,
    engine: Engine,
    renderer: Renderer,
    window: &'a Window,
}

impl<'a> Graphics<'a> {
    pub async fn new(window: &'a Window) -> Graphics<'a> {
        let size = window.inner_size();
        let physical_size = Size::new(size.width.max(1), size.height.max(1));
        let viewport = Viewport::with_physical_size(physical_size, window.scale_factor());

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: if cfg!(target_arch = "wasm32") {
                wgpu::Backends::GL
            } else {
                wgpu::Backends::PRIMARY
            },
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let adapter_features = adapter.features();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: adapter_features,
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };

        let engine = Engine::new(&adapter, &device, &queue, surface_format, None);
        let renderer = Renderer::new(&device, &engine, Font::default(), Pixels::from(16));

        Self {
            surface,
            device,
            queue,
            config,
            viewport,
            engine,
            renderer,
            window,
        }
    }

    fn window(&self) -> &Window {
        self.window
    }
}

pub struct App<'a> {
    pub graphics: Graphics<'a>,
    pub data: Option<AppData>,
    pub surface_configured: bool,
}

impl App<'_> {
    fn resize(&mut self, physical_size: Size<u32>) {
        // Ensure that the requested size will never be larger than the maximum texture dimension
        let max_dimension = self.graphics.device.limits().max_texture_dimension_2d;
        let physical_size = Size {
            width: physical_size.width.min(max_dimension),
            height: physical_size.height.min(max_dimension),
        };

        // Don't configure unless the size is valid
        if physical_size.width > 0 && physical_size.height > 0 {
            self.graphics.viewport =
                Viewport::with_physical_size(physical_size, self.graphics.window().scale_factor());
            self.graphics.config.width = physical_size.width;
            self.graphics.config.height = physical_size.height;
            self.graphics
                .surface
                .configure(&self.graphics.device, &self.graphics.config);

            // Resize the depth texture as well
            if let Some(data) = &mut self.data {
                data.scene.depth_texture =
                    Texture::depth_texture(&self.graphics.device, &physical_size);
            }

            // Mark the surface as being configured
            self.surface_configured = true;
        }
    }
}

pub struct AppData {
    scene: Scene,
    state: program::State<Controls>,
    cursor: Option<PhysicalPosition<f64>>,
    debug: Debug,
}

impl App<'_> {
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let Some(AppData {
            scene,
            state,
            debug,
            ..
        }) = &mut self.data
        else {
            return Ok(());
        };

        state.queue_message(PolybladeMessage::Tick(Instant::now()));

        let program = state.program();
        let program_state = &program.state;
        {
            let primitive =
                PolyhedronPrimitive::new(program_state.model.clone(), program_state.render.clone());
            let moments = primitive.moment_vertices();

            // Write barycentric and side data if a change in structure occurred
            if scene.moment_buf.len() != moments.len() {
                scene
                    .moment_buf
                    .resize(&self.graphics.device, moments.len());

                let shapes = primitive.model.polyhedron.shape_vertices();
                //log::error!("shapes: {shapes:?}");
                scene.shape_buf.resize(&self.graphics.device, shapes.len());
                scene.shape_buf.write_slice(&self.graphics.queue, &shapes);
            }

            // Write position and color data
            scene.moment_buf.write_slice(&self.graphics.queue, &moments);

            // Write Model Uniforms
            scene.model_buf.write_data(
                &self.graphics.queue,
                &ModelUniforms {
                    model_mat: primitive.model.transform,
                    view_projection_mat: primitive
                        .render
                        .camera
                        .build_view_proj_mat(self.graphics.viewport.logical_size()),
                },
            );
            // Write Frag Uniforms
            scene.frag_buf.write_data(
                &self.graphics.queue,
                &FragUniforms::new(primitive.render.line_thickness, 1.0),
            );
            self.graphics.window.request_redraw();
        }

        let output = self.graphics.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.graphics
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            // We clear the frame
            let mut render_pass = scene.clear(&view, &mut encoder, program.background_color());

            // Ignore the whole first polygon if we're in schlegel mode
            let starting_vertex = if program.state.render.schlegel {
                // Determines how many vertices are actually used to render the polygon
                program.state.model.polyhedron.starting_vertex()
            } else {
                0
            } as u32;

            // Draw the scene
            scene.draw(starting_vertex, &mut render_pass);
        }

        // And then iced on top
        self.graphics.renderer.present(
            &mut self.graphics.engine,
            &self.graphics.device,
            &self.graphics.queue,
            &mut encoder,
            None,
            output.texture.format(),
            &view,
            &self.graphics.viewport,
            &debug.overlay(),
        );

        // Then we submit the work
        self.graphics.engine.submit(&self.graphics.queue, encoder);
        output.present();

        // Update the mouse cursor interaction
        self.graphics
            .window
            .set_cursor(mouse_interaction(state.mouse_interaction()));

        Ok(())
    }
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let Graphics {
            device,
            config,
            viewport,
            renderer,
            ..
        } = &mut self.graphics;
        // Initialize scene and GUI controls
        let scene = Scene::new(device, config.format, &viewport.physical_size());
        let controls = Controls::new();
        // Initialize iced
        let mut debug = Debug::new();
        let state = program::State::new(controls, viewport.logical_size(), renderer, &mut debug);
        // You should change this if you want to render continuously
        event_loop.set_control_flow(ControlFlow::Poll);
        self.data = Some(AppData {
            scene,
            state,
            cursor: None,
            debug,
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        {
            match &event {
                WindowEvent::RedrawRequested => {
                    self.graphics.window().request_redraw();
                    if !self.surface_configured {
                        return;
                    }

                    match self.render() {
                        Ok(_) => {}
                        // Reconfigure the surface if it's lost or outdated
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            self.resize(self.graphics.viewport.physical_size());
                        }
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            log::error!("OutOfMemory");
                            event_loop.exit();
                        }

                        // This happens when the a frame takes too long to present
                        Err(wgpu::SurfaceError::Timeout) => {
                            log::warn!("Surface timeout")
                        }
                    }
                }
                WindowEvent::Resized(physical_size) => {
                    let size = Size {
                        width: physical_size.width,
                        height: physical_size.height,
                    };
                    self.resize(size);
                }
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    if event.state.is_pressed() {
                        let Some(key) = &event.text else {
                            return;
                        };

                        let message = if key.as_str() == key.to_uppercase().as_str() {
                            use PresetMessage::*;
                            match key.to_lowercase().as_str() {
                                // Presets
                                "t" => Some(Pyramid(3)),
                                "c" => Some(Prism(4)),
                                "o" => Some(Octahedron),
                                "d" => Some(Dodecahedron),
                                "i" => Some(Icosahedron),
                                _ => None,
                            }
                            .map(PolybladeMessage::Preset)
                        } else {
                            use ConwayMessage::*;
                            match key.as_str() {
                                // Operations
                                "e" => Some(Expand),
                                "d" => Some(Dual),
                                "s" => Some(SplitVertex(0)),
                                "k" => Some(Kis),
                                "j" => Some(Join),
                                "a" => Some(Ambo),
                                "t" => Some(Truncate),
                                "b" => Some(Bevel),
                                "c" => Some(Chamfer),
                                _ => None,
                            }
                            .map(PolybladeMessage::Conway)
                        };

                        if let (Some(message), Some(AppData { state, .. })) =
                            (message, &mut self.data)
                        {
                            state.queue_message(message);
                        }
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    if let Some(AppData { cursor, .. }) = &mut self.data {
                        *cursor = Some(*position);
                    }
                }
                _ => {}
            }
        }

        let Some(AppData {
            state,
            cursor,
            debug,
            ..
        }) = &mut self.data
        else {
            return;
        };

        // Map window event to iced event
        if let Some(event) = window_event(
            event,
            self.graphics.window().scale_factor(),
            ModifiersState::default(),
        ) {
            state.queue_event(event);
        }

        // If there are events pending
        if !state.is_queue_empty() {
            // We update iced
            let _ = state.update(
                self.graphics.viewport.logical_size(),
                cursor
                    .map(|p| cursor_position(p, self.graphics.viewport.scale_factor()))
                    .map(mouse::Cursor::Available)
                    .unwrap_or(mouse::Cursor::Unavailable),
                &mut self.graphics.renderer,
                &Theme::SolarizedLight,
                &renderer::Style::default(),
                &mut Clipboard::unconnected(),
                debug,
            );

            // and request a redraw
            self.graphics.window.request_redraw();
        }
    }
}
