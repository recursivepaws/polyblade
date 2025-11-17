use std::collections::HashMap;

use crate::render::{
    message::ColorMethodMessage,
    pipeline::{MomentVertex, ShapeVertex},
    state::{ModelState, RenderState},
};
use ultraviolet::{Vec3, Vec4};

#[derive(Debug)]
pub struct PolyhedronPrimitive {
    pub model: ModelState,
    pub render: RenderState,
}

impl PolyhedronPrimitive {
    pub fn new(model: ModelState, render: RenderState) -> Self {
        Self { model, render }
    }

    /* #[allow(dead_code)]
    pub fn surface_area(&self, face_index: usize) -> f32 {
        let positions: Vec<Vec3> = self.model.polyhedron.cycles[face_index]
            .iter()
            .map(|i| self.model.polyhedron.positions[i])
            .collect();
        let mut area = 0.0;
        for i in 0..positions.len() / 3 {
            let j = i * 3;
            let a = (positions[j] - positions[j + 1]).mag();
            let b = (positions[j + 1] - positions[j + 2]).mag();
            let c = (positions[j + 2] - positions[j]).mag();
            let s = (a + b + c) / 2.0;
            area += (s * (s - a) * (s - b) * (s - c)).sqrt();
        }
        area
    } */

    // All the vertices that will change moment to moment
    /*     pub fn moment_vertices(&self) -> Vec<MomentVertex> {
        let polyhedron = &self.model.polyhedron;
        let colors = &self.render.picker.palette.colors;

        match self.render.method {
            ColorMethodMessage::Vertex => todo!(),
            ColorMethodMessage::Edge => todo!(),
            ColorMethodMessage::Polygon => {
                // Polygon side count -> color
                let color_map: HashMap<usize, Vec4> =
                    polyhedron.cycles.iter().fold(HashMap::new(), |mut acc, c| {
                        if !acc.contains_key(&c.len()) {
                            acc.insert(c.len(), colors[acc.len() % colors.len()].into());
                        }
                        acc
                    });
                polyhedron
                    .cycles
                    .iter()
                    .map(|cycle| {
                        let color = *color_map.get(&cycle.len()).unwrap();
                        let positions: Vec<Vec3> =
                            cycle.iter().map(|&c| polyhedron.positions[&c]).collect();

                        match cycle.len() {
                            3 => positions
                                .iter()
                                .map(|&position| MomentVertex::new(position, color))
                                .collect(),
                            4 => [0usize, 1, 2, 2, 3, 0]
                                .iter()
                                .map(|&i| positions[i])
                                .map(|position| MomentVertex::new(position, color))
                                .collect(),
                            _ => {
                                let centroid: Vec3 =
                                    positions.iter().fold(Vec3::zero(), |a, &b| a + b)
                                        / positions.len() as f32;
                                (0..cycle.len())
                                    .map(|i| {
                                        vec![
                                            positions[i],
                                            centroid,
                                            positions[(i + 1) % positions.len()],
                                        ]
                                        .into_iter()
                                        .map(|position| MomentVertex::new(position, color))
                                        .collect()
                                    })
                                    .collect::<Vec<Vec<MomentVertex>>>()
                                    .concat()
                            }
                        }
                    })
                    .collect::<Vec<Vec<MomentVertex>>>()
                    .concat()
            }
            ColorMethodMessage::Face => todo!(),
        }
    } */

    /* pub fn shape_vertices(&self) -> Vec<ShapeVertex> {
        let barycentric = [Vec3::unit_x(), Vec3::unit_y(), Vec3::unit_z()];
        self.model
            .polyhedron
            .cycles
            .iter()
            .map(|cycle| {
                let sides: Vec4 = match cycle.len() {
                    3 => Vec3::new(1.0, 1.0, 1.0),
                    4 => Vec3::new(1.0, 0.0, 1.0),
                    _ => Vec3::new(0.0, 1.0, 0.0),
                }
                .into();

                let b_shapes: Vec<ShapeVertex> = barycentric
                    .iter()
                    .map(|&b| ShapeVertex {
                        barycentric: b.into(),
                        sides,
                    })
                    .collect();

                match cycle.len() {
                    3 => b_shapes.clone(),
                    4 => (0..6)
                        .map(|i| ShapeVertex {
                            barycentric: barycentric[i % 3].into(),
                            sides,
                        })
                        .collect(),
                    _ => vec![b_shapes; cycle.len()].concat(),
                }
            })
            .collect::<Vec<Vec<ShapeVertex>>>()
            .concat()
    } */
}
