use ultraviolet::{Mat4, Vec3, Vec4};

#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct MomentVertex {
    pub position: Vec3,
    pub color: Vec4,
    pub _padding: f32,
    // #[cfg(target_arch = "wasm32")]
}

impl MomentVertex {
    pub fn new(position: Vec3, color: Vec4) -> MomentVertex {
        Self {
            position,
            _padding: 123.4,
            color,
        }
    }
}

#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
//#[repr(C, align(16))]
#[repr(C)]
pub struct ShapeVertex {
    pub barycentric: Vec4,
    pub sides: Vec4,
}

#[derive(Copy, Default, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
//#[repr(C, align(16))]
#[repr(C)]
pub struct ModelUniforms {
    pub(crate) model_mat: Mat4,
    pub(crate) view_projection_mat: Mat4,
}

#[derive(Copy, Default, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
//#[repr(C, align(16))]
#[repr(C)]
pub struct FragUniforms {
    pub(crate) line_thickness: f32,
    pub(crate) line_mode: f32,
    pub _padding: [f32; 2],
}
impl FragUniforms {
    pub fn new(line_thickness: f32, line_mode: f32) -> Self {
        Self {
            line_thickness,
            line_mode,
            _padding: [123.4; 2],
        }
    }
}
