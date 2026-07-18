mod types;
use iced::widget::shader::wgpu::{self};
pub use types::*;

pub enum BufferKind {
    Uniform,
    Index,
    Vertex,
}

impl From<BufferKind> for wgpu::BufferUsages {
    fn from(val: BufferKind) -> Self {
        match val {
            BufferKind::Uniform => wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            BufferKind::Index => wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            BufferKind::Vertex => wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        }
    }
}

// A custom buffer container for dynamic resizing.
pub struct Buffer {
    raw: wgpu::Buffer,
    label: &'static str,
    usage: wgpu::BufferUsages,
    size_of_type: u64,
    count: usize,
}

impl Buffer {
    pub fn len(&self) -> usize {
        self.count
    }

    pub fn new<T>(device: &wgpu::Device, label: &'static str, kind: BufferKind) -> Self {
        let size_of_type = std::mem::size_of::<T>() as u64;
        let usage = kind.into();
        Self {
            raw: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: size_of_type,
                usage,
                mapped_at_creation: false,
            }),
            label,
            usage,
            size_of_type,
            count: 1,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, new_count: usize) {
        self.raw = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(self.label),
            size: self.size_of_type * new_count as u64,
            usage: self.usage,
            mapped_at_creation: false,
        });
        self.count = new_count;
    }

    pub fn write_data<T: bytemuck::Pod>(&mut self, queue: &wgpu::Queue, data: &T) {
        queue.write_buffer(&self.raw, 0, bytemuck::bytes_of(data));
    }

    pub fn write_slice<A: bytemuck::NoUninit>(&mut self, queue: &wgpu::Queue, slice: &[A]) {
        queue.write_buffer(&self.raw, 0, bytemuck::cast_slice(slice));
    }

    pub fn raw_slice(&self) -> wgpu::BufferSlice<'_> {
        self.raw.slice(..)
    }

    pub fn binding_resource(&self) -> wgpu::BindingResource<'_> {
        self.raw.as_entire_binding()
    }
}
