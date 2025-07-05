use dioxus::prelude::*;
use polyblade::graphics::{Vertex, WGPUInstance};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use ultraviolet::Vec3;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::wgt::{CommandEncoderDescriptor, TextureViewDescriptor};
use wgpu::PrimitiveTopology::TriangleList;
use wgpu::{
    include_wgsl, BlendState, Buffer, BufferUsages, Color, ColorTargetState, ColorWrites,
    FragmentState, LoadOp, Operations, PipelineLayoutDescriptor, PrimitiveState,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    StoreOp, VertexState,
};

#[cfg(target_arch = "wasm32")]
use {log::info, wgpu::SurfaceTarget::Canvas};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[derive(Debug, Clone, EnumIter, PartialEq, Display)]
enum Platonic {
    Tetrahedron,
    Hexahedron,
    Octahedron,
    Dodecahedron,
    Icosahedron,
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    wasm_logger::init(wasm_logger::Config::default());
    launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Stylesheet { href: MAIN_CSS }
        document::Stylesheet { href: TAILWIND_CSS }
        Router::<Route> {}
    }
}

/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    rsx! {
        div { class: "main-div",
            div { class: "menu-bar",
                div { class: "menu-group",
                    div { class: "menu-btn", "File" }
                    div { class: "dropdown",
                        div { class: "item",
                            "Open"
                            span { class: "shortcut", "#O" }
                        }

                        div { class: "item has-sub",
                            "Recent"
                            div { class: "submenu",
                                for preset in Platonic::iter() {
                                    div { class: "item", "file_{preset}.doc" }
                                }
                            }
                        }
                    }
                }
            }
            Outlet::<Route> {}
        }
    }
}

/// Home page
#[component]
fn Home() -> Element {
    rsx! {
        Line {}
    }
}

#[component]
pub fn Line() -> Element {
    let triangle: Triangle = vec![
        Vertex {
            position: Vec3::new(0.0, 0.5, 0.0),
            color: Vec3::new(1.0, 0.0, 0.0),
        },
        Vertex {
            position: Vec3::new(-0.5, -0.5, 0.0),
            color: Vec3::new(0.0, 1.0, 0.0),
        },
        Vertex {
            position: Vec3::new(0.5, -0.5, 0.0),
            color: Vec3::new(0.0, 0.0, 1.0),
        },
    ];

    #[cfg(target_arch = "wasm32")]
    use_effect(move || {
        if let Some(el) = polyblade::get_canvas(&"wgpu-canvas") {
            let tri = triangle.clone();

            spawn(async move {
                let gpu = WGPUInstance::new(Canvas(el)).await;
                info!("wgpu_instance created");
                let renderer = Renderer::new(&gpu, &tri);
                renderer.render(&gpu);
            });
        } else {
            info!("failed to find canvas.")
        }
    });

    rsx! {
        div { class: "canvas-div",
            canvas { id: "wgpu-canvas", width: 1000, height: 1000 }
            canvas {
                id: "backup-canvas",
                background_color: "green",
                width: 1000,
                height: 1000,
            }
        }
    }
}

type Triangle = Vec<Vertex>;

struct Renderer {
    vertex_buffer: Buffer,
    pipeline: RenderPipeline,
}

impl Renderer {
    pub fn new(gpu: &WGPUInstance, model: &Triangle) -> Self {
        let vertex_buffer = gpu.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buff"),
            contents: bytemuck::cast_slice(&model),
            usage: BufferUsages::VERTEX,
        });

        let shader = gpu
            .device
            .create_shader_module(include_wgsl!("shaders/shader.wgsl"));

        let layout = gpu
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let pipeline = gpu
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[Vertex::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(ColorTargetState {
                        format: gpu.config.format,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: PrimitiveState {
                    topology: TriangleList,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: Default::default(),
                multiview: None,
                cache: None,
            });

        Self {
            vertex_buffer,
            pipeline,
        }
    }

    pub fn render(&self, gpu: &WGPUInstance) {
        let frame = gpu.surface.get_current_texture().unwrap();

        let view = frame.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = gpu
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..3, 0..1);
        }

        gpu.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
