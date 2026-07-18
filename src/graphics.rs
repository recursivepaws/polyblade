use log::info;
use wgpu::DeviceDescriptor;

#[cfg(target_arch = "wasm32")]
use wgpu::SurfaceTarget::Canvas;

use wgpu::{
    Device, Instance, PowerPreference, Queue, RequestAdapterOptions, Surface,
    SurfaceConfiguration, SurfaceTarget, TextureFormat,
};

pub struct WGPUInstance<'window> {
    pub surface: Surface<'window>,
    pub config: SurfaceConfiguration,
    pub device: Device,
    pub queue: Queue,
    /// Format render pipelines and per-frame views should target: the surface
    /// format itself when it's sRGB, otherwise an sRGB view format of it.
    pub render_format: TextureFormat,
}

impl<'window> WGPUInstance<'window> {
    pub async fn new(target: SurfaceTarget<'window>) -> Self {
        #[allow(unused_mut)]
        let mut width = 1;
        #[allow(unused_mut)]
        let mut height = 1;

        #[cfg(target_arch = "wasm32")]
        if let Canvas(canvas) = &target {
            width = canvas.width();
            height = canvas.height();
        }

        let instance = Instance::default();
        info!("wgpu instance created");

        let surface = instance.create_surface(target).unwrap();
        info!("wgpu surface created");

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .unwrap();
        info!("wgpu adapter created");

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("Device"),
                required_features: Default::default(),
                required_limits: Default::default(),
                memory_hints: Default::default(),
                trace: Default::default(),
            })
            .await
            .unwrap();
        info!("wgpu device and queue created.");

        let mut config = surface.get_default_config(&adapter, width, height).unwrap();

        // The palette colors are converted to linear space on upload, so the
        // render target must be sRGB for output to re-encode correctly.
        let render_format = if config.format.is_srgb() {
            config.format
        } else {
            let srgb = config.format.add_srgb_suffix();
            if srgb == config.format {
                // No sRGB variant exists; render in the surface format as-is.
                config.format
            } else {
                config.view_formats.push(srgb);
                srgb
            }
        };

        surface.configure(&device, &config);

        Self {
            surface,
            config,
            device,
            queue,
            render_format,
        }
    }
}
