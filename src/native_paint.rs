use crate::{render::driver::RenderDriver, Instant};
use dioxus_native::{CustomPaintCtx, CustomPaintSource, DeviceHandle, TextureHandle};
use wgpu::{
    Device, Extent3d, Queue, Texture, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureViewDescriptor,
};

pub struct PolybladePaintSource {
    state: State,
}

enum State {
    Suspended,
    Active(Box<ActiveRenderer>),
}

struct TextureAndHandle {
    texture: Texture,
    handle: TextureHandle,
}

struct ActiveRenderer {
    device: Device,
    queue: Queue,
    driver: RenderDriver,
    displayed_texture: Option<TextureAndHandle>,
    next_texture: Option<TextureAndHandle>,
}

impl PolybladePaintSource {
    pub fn new() -> Self {
        Self {
            state: State::Suspended,
        }
    }
}

impl Default for PolybladePaintSource {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomPaintSource for PolybladePaintSource {
    fn resume(&mut self, device_handle: &DeviceHandle) {
        let device = device_handle.device.clone();
        let queue = device_handle.queue.clone();
        // Blitz's compositor expects Rgba8Unorm textures; we render through an
        // sRGB view of them so shader output is gamma-encoded correctly.
        let driver = RenderDriver::new(&device, TextureFormat::Rgba8UnormSrgb, 1, 1);
        self.state = State::Active(Box::new(ActiveRenderer {
            device,
            queue,
            driver,
            displayed_texture: None,
            next_texture: None,
        }));
    }

    fn suspend(&mut self) {
        self.state = State::Suspended;
    }

    fn render(
        &mut self,
        ctx: CustomPaintCtx<'_>,
        width: u32,
        height: u32,
        _scale: f64,
    ) -> Option<TextureHandle> {
        if width == 0 || height == 0 {
            return None;
        }
        let State::Active(state) = &mut self.state else {
            return None;
        };
        state.render(ctx, width, height)
    }
}

impl ActiveRenderer {
    fn render(
        &mut self,
        mut ctx: CustomPaintCtx<'_>,
        width: u32,
        height: u32,
    ) -> Option<TextureHandle> {
        if let Some(next) = &self.next_texture {
            if next.texture.width() != width || next.texture.height() != height {
                ctx.unregister_texture(self.next_texture.take().unwrap().handle);
            }
        }

        let texture_and_handle = match &self.next_texture {
            Some(next) => next,
            None => {
                let texture = create_texture(&self.device, width, height);
                let handle = ctx.register_texture(texture.clone());
                self.next_texture = Some(TextureAndHandle { texture, handle });
                self.next_texture.as_ref().unwrap()
            }
        };

        let view = texture_and_handle.texture.create_view(&TextureViewDescriptor {
            format: Some(TextureFormat::Rgba8UnormSrgb),
            ..Default::default()
        });
        let handle = texture_and_handle.handle.clone();

        self.driver.resize(&self.device, width, height);
        self.driver.tick(Instant::now());
        self.driver.draw(&self.device, &self.queue, &view);

        std::mem::swap(&mut self.next_texture, &mut self.displayed_texture);
        Some(handle)
    }
}

fn create_texture(device: &Device, width: u32, height: u32) -> Texture {
    device.create_texture(&TextureDescriptor {
        label: Some("Polyblade Paint Texture"),
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::RENDER_ATTACHMENT
            | TextureUsages::TEXTURE_BINDING
            | TextureUsages::COPY_SRC,
        view_formats: &[TextureFormat::Rgba8UnormSrgb],
    })
}
