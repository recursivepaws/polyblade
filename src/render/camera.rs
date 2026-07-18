use iced::Size;
use ultraviolet::{Mat4, Vec3, Vec4};

#[derive(Copy, Debug, Clone)]
pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov_y: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: Vec3::new(0.0, 5.0, 5.0),
            target: Vec3::zero(),
            up: Vec3::unit_y(),
            fov_y: 0.4,
            near: 0.1,
            far: 10.0,
        }
    }
}

pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::new(
    Vec4::new(1.0, 0.0, 0.0, 0.0),
    Vec4::new(0.0, 1.0, 0.0, 0.0),
    Vec4::new(0.0, 0.0, 0.5, 0.0),
    Vec4::new(0.0, 0.0, 0.5, 1.0),
);

impl Camera {
    pub fn build_view_proj_mat(&self, bounds: Size<f32>) -> Mat4 {
        let aspect_ratio = bounds.width / bounds.height;
        let view = Mat4::look_at(self.eye, self.target, self.up);
        let h = f32::cos(0.5 * self.fov_y) / f32::sin(0.5 * self.fov_y);
        let w = h / aspect_ratio;
        let r = self.far / (self.near - self.far);
        let proj = Mat4::new(
            Vec4::new(w, 0.0, 0.0, 0.0),
            Vec4::new(0.0, h, 0.0, 0.0),
            Vec4::new(0.0, 0.0, r, -1.0),
            Vec4::new(0.0, 0.0, r * self.near, 0.0),
        );

        OPENGL_TO_WGPU_MATRIX * proj * view
    }

    #[allow(dead_code)]
    pub fn position(&self) -> Vec4 {
        Vec4::from(self.eye)
    }
}
