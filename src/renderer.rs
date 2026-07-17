use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    include_wgsl, BlendState, Buffer, BufferUsages, Color, ColorTargetState, ColorWrites,
    CommandEncoderDescriptor, Device, FragmentState, LoadOp, MultisampleState, Operations,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, StoreOp, TextureFormat,
    TextureView, TextureViewDescriptor, VertexState,
};

use crate::graphics::{Vertex, WGPUInstance};

pub type Triangle = Vec<Vertex>;

pub struct Renderer {
    vertex_buffer: Buffer,
    pipeline: RenderPipeline,
}

impl Renderer {
    pub fn new(device: &Device, format: TextureFormat, model: &Triangle) -> Self {
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buff"),
            contents: bytemuck::cast_slice(&model),
            usage: BufferUsages::VERTEX,
        });

        let shader = device.create_shader_module(include_wgsl!("shaders/shader.wgsl"));

        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
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
                    format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            vertex_buffer,
            pipeline,
        }
    }

    pub fn render_surface(&self, gpu: &WGPUInstance) {
        let frame = gpu.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        self.render_to_view(&gpu.device, &gpu.queue, &view);
        frame.present();
    }

    pub fn render_to_view(&self, device: &Device, queue: &Queue, view: &TextureView) {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Command Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..3, 0..1);
        }

        queue.submit(Some(encoder.finish()));
    }
}
