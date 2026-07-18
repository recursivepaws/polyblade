use crate::{
    render::{
        message::{PolybladeMessage, ProcessMessage},
        pipeline::{FragUniforms, ModelUniforms, PolyhedronPrimitive, Scene},
        state::AppState,
    },
    Instant,
};
use wgpu::{CommandEncoderDescriptor, Device, Queue, TextureFormat, TextureView};

/// Owns the app state and wgpu scene, reproducing the per-frame flow of the
/// old iced app: tick the simulation, upload vertex/uniform data, then draw.
pub struct RenderDriver {
    pub state: AppState,
    scene: Scene,
    size: (u32, u32),
}

impl RenderDriver {
    pub fn new(device: &Device, format: TextureFormat, width: u32, height: u32) -> Self {
        Self {
            state: AppState::default(),
            scene: Scene::new(device, format, (width, height)),
            size: (width, height),
        }
    }

    /// Recreates the render-target-sized textures when the target changes size.
    pub fn resize(&mut self, device: &Device, width: u32, height: u32) {
        if self.size != (width, height) {
            self.size = (width, height);
            self.scene.resize(device, (width, height));
        }
    }

    /// Advances physics and the model transform, exactly like the old Tick message.
    pub fn tick(&mut self, now: Instant) {
        for msg in crate::render::message::drain_messages() {
            msg.process(&mut self.state);
        }
        PolybladeMessage::Tick(now).process(&mut self.state);
    }

    pub fn draw(&mut self, device: &Device, queue: &Queue, view: &TextureView) {
        let primitive =
            PolyhedronPrimitive::new(self.state.model.clone(), self.state.render.clone());
        let moments = primitive.moment_vertices();

        // Write barycentric and side data if a change in structure occurred
        if self.scene.moment_buf.len() != moments.len() {
            self.scene.moment_buf.resize(device, moments.len());

            let shapes = primitive.model.polyhedron.shape_vertices();
            self.scene.shape_buf.resize(device, shapes.len());
            self.scene.shape_buf.write_slice(queue, &shapes);
        }

        // Write position and color data
        self.scene.moment_buf.write_slice(queue, &moments);

        let (width, height) = self.size;
        // Write Model Uniforms
        self.scene.model_buf.write_data(
            queue,
            &ModelUniforms {
                model_mat: primitive.model.transform,
                view_projection_mat: primitive
                    .render
                    .camera
                    .build_view_proj_mat(width as f32, height as f32),
            },
        );
        // Write Frag Uniforms
        self.scene.frag_buf.write_data(
            queue,
            &FragUniforms::new(primitive.render.line_thickness, 1.0),
        );

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        {
            let mut render_pass = self.scene.clear(
                view,
                &mut encoder,
                self.state.render.background_color.into(),
            );

            // Ignore the whole first polygon if we're in schlegel mode
            let starting_vertex = if self.state.render.schlegel {
                self.state.model.polyhedron.starting_vertex()
            } else {
                0
            } as u32;

            self.scene.draw(starting_vertex, &mut render_pass);
        }
        queue.submit(Some(encoder.finish()));
    }
}
