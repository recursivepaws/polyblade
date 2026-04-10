use crate::{
    polyhedron::{Polyhedron, Transaction},
    render::camera::Camera,
    Instant,
};
use iced::{Color, Task};
use std::fmt::Display;
use strum_macros::{Display, EnumIter};

use crate::render::state::{AppState, ColorPickerState, ModelState, RenderState};

#[derive(Debug, Clone, Display)]
pub enum PolybladeMessage {
    Tick(Instant),
    Preset(PresetMessage),
    Conway(ConwayMessage),
    Render(RenderMessage),
    OpenWiki(String),
}

#[derive(Debug, Clone, EnumIter)]
pub enum PresetMessage {
    Prism(usize),
    AntiPrism(usize),
    Pyramid(usize),
    Octahedron,
    Dodecahedron,
    Icosahedron,
}

impl Display for PresetMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use PresetMessage::*;
        let value = match self {
            Prism(n) => match n {
                3 => "Triangular",
                4 => "Cube",
                5 => "Pentagonal",
                6 => "Hexagonal",
                7 => "Heptagonal",
                8 => "Octagonal",
                _ => "?",
            },
            AntiPrism(n) => match n {
                2 => "Digonal",
                3 => "Triangular",
                4 => "Square",
                5 => "Pentagonal",
                6 => "Hexagonal",
                7 => "Heptagonal",
                8 => "Octagonal",
                _ => "?",
            },
            Pyramid(n) => match n {
                3 => "Tetrahedron",
                4 => "Square",
                5 => "Pentagonal",
                6 => "Hexagonal",
                7 => "Heptagonal",
                8 => "Octagonal",
                _ => "?",
            },
            _ => &format!("{self:?}"),
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, EnumIter, Display)]
pub enum ConwayMessage {
    // 1
    Dual,
    // 2
    Join,
    Ambo,
    // 3
    Kis,
    // Needle,
    // Zip,
    SplitVertex(usize),
    Truncate,
    // 4
    //Ortho,
    Expand,
    // 5
    // Gyro,
    Snub,
    // // 6
    // Meta,
    Bevel,

    Chamfer,
}

#[derive(Debug, Clone)]
pub enum RenderMessage {
    Schlegel(bool),
    Rotating(bool),
    FovChanged(f32),
    ZoomChanged(f32),
    SpeedChanged(f32),
    LineThickness(f32),
    ColorMethod(ColorMethodMessage),
    ColorPicker(ColorPickerMessage),
}

impl Display for RenderMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use RenderMessage::*;
        let value = match &self {
            ColorMethod(method) => method.to_string(),
            _ => {
                format!("{self:?}")
            }
        };
        f.write_str(&value)
    }
}

#[derive(Debug, Clone, EnumIter, Display)]
pub enum ColorMethodMessage {
    Vertex,
    Edge,
    Polygon,
    Face,
}

impl From<ColorMethodMessage> for f32 {
    fn from(val: ColorMethodMessage) -> Self {
        match val {
            ColorMethodMessage::Vertex => 0.0,
            ColorMethodMessage::Edge => 1.0,
            ColorMethodMessage::Polygon => 2.0,
            ColorMethodMessage::Face => 2.0,
        }
    }
}

#[derive(Debug, Clone, EnumIter, Display)]
pub enum ColorPickerMessage {
    ChangeNumber(i16),
    ChooseColor(usize),
    SubmitColor(Color),
    CancelColor,
}

#[derive(Debug, Clone, EnumIter, Display)]
pub enum ModelMessage {
    ScaleChanged(f32),
}

pub trait ProcessMessage<T> {
    fn process(&self, state: &mut T) -> Task<PolybladeMessage>;
}

impl ProcessMessage<ModelState> for PresetMessage {
    fn process(&self, state: &mut ModelState) -> Task<PolybladeMessage> {
        state.polyhedron = Polyhedron::preset(self);
        Task::none()
    }
}

impl ProcessMessage<ModelState> for ConwayMessage {
    fn process(&self, state: &mut ModelState) -> Task<PolybladeMessage> {
        state
            .polyhedron
            .transactions
            .push(Transaction::Conway(self.clone()));
        Task::none()
    }
}

impl ProcessMessage<RenderState> for RenderMessage {
    fn process(&self, state: &mut RenderState) -> Task<PolybladeMessage> {
        use RenderMessage::*;
        match &self {
            Schlegel(schlegel) => {
                state.schlegel = *schlegel;
                if *schlegel {
                    state.camera.fov_y = 2.9;
                    state.zoom = 1.1;
                } else {
                    state.camera = Camera::default();
                }
                Task::none()
            }
            Rotating(rotating) => {
                state.rotating = *rotating;
                if !rotating {
                    state.rotation_duration = Instant::now().duration_since(state.start);
                } else {
                    state.start = Instant::now().checked_sub(state.rotation_duration).unwrap();
                }
                Task::none()
            }
            FovChanged(fov) => {
                state.camera.fov_y = *fov;
                Task::none()
            }
            ZoomChanged(zoom) => {
                state.zoom = *zoom;
                Task::none()
            }
            SpeedChanged(speed) => {
                state.speed = *speed;
                Task::none()
            }
            LineThickness(thickness) => {
                state.line_thickness = *thickness;
                Task::none()
            }
            ColorMethod(method) => {
                state.method = method.clone();
                Task::none()
            }
            ColorPicker(picker) => picker.process(&mut state.picker),
        }
    }
}

impl ProcessMessage<ColorPickerState> for ColorPickerMessage {
    fn process(&self, state: &mut ColorPickerState) -> Task<PolybladeMessage> {
        use ColorPickerMessage::*;
        match self {
            ChangeNumber(colors) => {
                state.colors = *colors;
            }
            ChooseColor(i) => {
                state.color_index = Some(*i);
                state.picked_color = state.palette.colors[*i].into();
            }
            SubmitColor(color) => {
                state.picked_color = *color;
                if let Some(i) = state.color_index {
                    state.palette.colors[i] = (*color).into();
                }
                state.color_index = None;
            }
            CancelColor => {
                state.color_index = None;
            }
        }
        Task::none()
    }
}

impl ProcessMessage<AppState> for PolybladeMessage {
    fn process(&self, state: &mut AppState) -> Task<PolybladeMessage> {
        //println!("processing message: {self:?} for state {state:?}");
        use PolybladeMessage::*;
        match self {
            Tick(time) => {
                if state.render.schlegel {
                    state.render.camera.eye =
                        state.model.polyhedron.face_centroid(0) * state.render.zoom;
                }

                // If the polyhedron has changed
                if state.info.conway != state.model.polyhedron.name {
                    // Recompute its Polydex entry
                    state.info = state.model.polyhedron.polydex_entry(&state.polydex);
                }

                state.update_state(*time);
                Task::none()
            }
            Preset(preset) => preset.process(&mut state.model),
            Conway(conway) => conway.process(&mut state.model),
            Render(render) => render.process(&mut state.render),
            OpenWiki(wiki) => {
                let _ = webbrowser::open(wiki).ok();
                Task::none()
            }
        }
    }
}
