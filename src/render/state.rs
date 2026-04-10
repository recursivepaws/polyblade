use crate::{
    polyhedron::Polyhedron,
    render::{
        camera::Camera,
        message::{ColorMethodMessage, PresetMessage},
        palette::Palette,
        polydex::{Entry, InfoBox, Polydex},
    },
    Instant,
};

use iced::{time::Duration, Color};
use std::{f32::consts::PI, io::Read as _};
use ultraviolet::Mat4;

#[derive(Debug, Default)]
pub struct AppState {
    pub model: ModelState,
    pub render: RenderState,
    pub polydex: Polydex,
    pub info: InfoBox,
}

#[derive(Debug, Clone)]
pub struct RenderState {
    pub camera: Camera,
    pub zoom: f32,
    pub speed: f32,
    pub start: Instant,
    pub frame: Instant,
    pub rotation_duration: Duration,
    pub rotating: bool,
    pub schlegel: bool,
    pub line_thickness: f32,
    pub method: ColorMethodMessage,
    pub picker: ColorPickerState,
    pub background_color: Color,
}

#[derive(Debug, Clone)]
pub struct ColorPickerState {
    pub palette: Palette,
    pub color_index: Option<usize>,
    pub picked_color: Color,
    pub colors: i16,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            camera: Camera::default(),
            zoom: 1.0,
            speed: 10.0,
            start: Instant::now(),
            frame: Instant::now(),
            rotation_duration: Duration::from_secs(0),
            rotating: true,
            schlegel: false,
            line_thickness: 2.0,
            method: ColorMethodMessage::Polygon,
            picker: ColorPickerState::default(),
            background_color: Color::WHITE,
        }
    }
}

impl Default for ColorPickerState {
    fn default() -> Self {
        Self {
            palette: Palette::clement(),
            color_index: None,
            picked_color: Color::from_rgba8(0, 0, 0, 1.0),
            colors: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModelState {
    pub polyhedron: Polyhedron,
    pub transform: Mat4,
}

impl Default for ModelState {
    fn default() -> Self {
        //log::error!("poly: {:?}", x.polyhedron);
        Self {
            //polyhedron: { Polyhedron::preset(&PresetMessage::Octahedron) },
            polyhedron: { Polyhedron::preset(&PresetMessage::Pyramid(3)) },
            transform: Mat4::identity(),
        }
    }
}

#[allow(dead_code)]
pub fn load_polydex() -> Result<Polydex, Box<dyn std::error::Error>> {
    let mut polydex = std::fs::File::open("assets/polydex.ron")?;
    let mut polydex_str = String::new();
    polydex.read_to_string(&mut polydex_str)?;
    let polydex: Vec<Entry> = ron::from_str(&polydex_str).map_err(|_| "Ron parsing error")?;
    Ok(polydex)
}

impl AppState {
    pub fn update_state(&mut self, time: Instant) {
        // Update the polyhedron using the difference in time between this and the previous frame
        let frame_difference = time.duration_since(self.render.frame).as_secs_f32();
        let framerate = 1.0 / 60.0;
        //sleep_ms(800);
        // Fraction of a second since the previous frame rendered
        let second = if frame_difference > 1.0 / 60.0 {
            // log::warn!("took more than 1/60th of a second to render that frame");
            framerate
        } else {
            frame_difference
        };

        self.model.polyhedron.update(self.render.speed, second);
        self.render.frame = time;

        let time = if self.render.rotating {
            time.duration_since(self.render.start)
        } else {
            self.render.rotation_duration
        };

        let time = time.as_secs_f32();
        self.model.transform = Mat4::default();
        if self.render.schlegel {
            self.model.transform = Mat4::identity();
        } else {
            self.model.transform = Mat4::from_scale(self.render.zoom)
                * Mat4::from_rotation_x(time / PI)
                * Mat4::from_rotation_y(time / PI * 1.1);
        }
    }
}
