pub struct Texture {
    pub view: wgpu::TextureView,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;
    /// 4x MSAA is guaranteed to be supported for the render formats we use.
    pub const SAMPLE_COUNT: u32 = 4;

    pub fn depth_texture(device: &wgpu::Device, target_size: (u32, u32)) -> Self {
        Self::render_attachment(device, "Depth Texture", Self::DEPTH_FORMAT, target_size)
    }

    /// Multisampled color target that gets resolved into the output texture.
    pub fn msaa_texture(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        target_size: (u32, u32),
    ) -> Self {
        Self::render_attachment(device, "MSAA Texture", format, target_size)
    }

    fn render_attachment(
        device: &wgpu::Device,
        label: &str,
        format: wgpu::TextureFormat,
        target_size: (u32, u32),
    ) -> Self {
        let size = wgpu::Extent3d {
            width: target_size.0.max(1),
            height: target_size.1.max(1),
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: Self::SAMPLE_COUNT,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[format],
        };

        let texture = device.create_texture(&desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self { view }
    }
}
