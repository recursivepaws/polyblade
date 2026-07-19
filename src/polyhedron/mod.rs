mod conway;
mod platonic;
mod render;
mod shape;
mod transaction;
use render::*;
use shape::*;
pub use transaction::*;

#[cfg(test)]
mod test;

use std::{collections::HashMap, time::Duration};

use crate::Instant;
use crate::render::{
    message::{ConwayMessage, PresetMessage},
    pipeline::{MomentVertex, ShapeVertex},
};
use ultraviolet::{Vec3, Vec4};

pub type VertexId = usize;

pub const SPEED_DAMPENING: f32 = 0.92;

/// Margin for the auto-fit Schlegel FOV so extremal vertices don't touch the viewport edge.
const SCHLEGEL_FOV_FILL: f32 = 0.85;

#[derive(Debug, Clone)]
pub struct Polyhedron {
    /// Conway Polyhedron Notation
    pub name: String,
    /// The shape we're rendering
    shape: Shape,
    /// Position data
    pub render: Render,
    /// Transaction queue
    pub transactions: Vec<Transaction>,
}

impl Polyhedron {
    pub fn shape_vertices(&self) -> Vec<ShapeVertex> {
        self.shape.cycles.shape_vertices()
    }
    pub fn starting_vertex(&self) -> VertexId {
        match self.shape.cycles[0].len() {
            3 => 3,
            4 => 6,
            n => n * 3,
        }
    }

    pub fn process_transactions(&mut self, _speed: f32) {
        if let Some(transaction) = self.transactions.first().cloned() {
            use Transaction::*;
            match transaction {
                Contraction(edges) => {
                    let Polyhedron {
                        shape,
                        render,
                        transactions,
                        ..
                    } = self;

                    let all_completed = !edges
                        .iter()
                        .map(|&[v, u]| render.spring_length([v, u]))
                        .any(|l| l > 0.05);

                    if all_completed {
                        // Contract them in the graph
                        shape.contract_edges(edges.clone());
                        render.contract_edges(edges);
                        transactions.remove(0);
                    }
                }
                Release(edges) => {
                    self.shape.release(&edges);
                    self.transactions.remove(0);
                }
                Conway(conway) => {
                    self.transactions.remove(0);
                    use ConwayMessage::*;
                    use Transaction::*;
                    let new_transactions = match conway {
                        Dual => {
                            // let edges = self.expand(false);
                            // vec![
                            //     Wait(Instant::now() + Duration::from_millis((65.0 * speed) as u64)),
                            //     Contraction(edges),
                            //     Name('d'),
                            // ]
                            todo!()
                        }
                        Join => {
                            // let edges = self.graph.kis(Option::None);
                            // vec![
                            //     //Wait(Instant::now() + Duration::from_secs(1)),
                            //     Release(edges),
                            //     Name('j'),
                            // ]
                            todo!()
                        }
                        Ambo => {
                            // let edges = self.shape.ambo();
                            // self.shape.recompute();
                            let edges = self.ambo();
                            vec![Contraction(edges), Name('a')]
                        }
                        Chamfer => {
                            self.chamfer();
                            vec![Name('c')]
                        }
                        Kis => {
                            // self.graph.kis(Option::None);
                            // vec![Name('k')]
                            todo!()
                        }
                        SplitVertex(n) => {
                            self.split_vertex(n);
                            self.shape.recompute();
                            vec![]
                        }
                        Truncate => {
                            // let mut operations = vec![];
                            // for v in self.shape.vertices() {
                            //     operations.extend(vec![
                            //         Wait(Instant::now() + Duration::from_millis(1000) * v as u32),
                            //         Conway(SplitVertex(v)),
                            //     ]);
                            // }
                            // [operations, vec![Name('t')]].concat()
                            self.truncate(0);
                            vec![Name('t')]
                        }
                        Expand => {
                            self.ambo_contract();
                            let edges = self.ambo();
                            // self.shape.expand(false);
                            vec![Contraction(edges), Name('e')]
                        }
                        Snub => {
                            // self.graph.expand(true);
                            // vec![Name('s')]
                            todo!()
                        }
                        Bevel => {
                            vec![
                                Conway(Truncate),
                                Wait(Instant::now() + Duration::from_millis(500)),
                                Conway(Ambo),
                                Name('b'),
                            ]
                        }
                    };
                    self.render.new_capacity(self.shape.order());
                    self.transactions = [new_transactions, self.transactions.clone()].concat();
                }
                Name(c) => {
                    if c == 'b' {
                        self.name = self.name[2..].to_string();
                    }
                    if c == 'd' && &self.name[0..1] == "d" {
                        self.name = self.name[1..].to_string();
                    } else {
                        self.name = format!("{c}{}", self.name);
                    }
                    self.transactions.remove(0);
                }
                ShortenName(n) => {
                    self.name = self.name[n..].to_string();
                    self.transactions.remove(0);
                }
                Wait(instant) => {
                    if Instant::now() > instant {
                        self.transactions.remove(0);
                    }
                }
                None => {}
            };
        }
    }

    pub fn update(&mut self, speed: f32, second: f32) {
        self.render.update(speed, second);
        self.apply_spring_forces(speed, second);
        self.process_transactions(speed);
    }

    fn apply_spring_forces(&mut self, speed: f32, second: f32) {
        let Polyhedron {
            shape,
            render,
            transactions,
            ..
        } = self;
        //let diameter = shape.diameter();
        let diameter_spring_length = render.edge_length * 2.0;

        // If we're contracting, we end up working with a more narrow set of edges
        let (edges, contracting): (std::slice::Iter<[VertexId; 2]>, bool) =
            if let Some(Transaction::Contraction(edges)) = transactions.first() {
                (edges.iter(), true)
            } else {
                (shape.springs.iter(), false)
            };

        for &[v, u] in edges {
            let spring_length = render.spring_length([v, u]);
            if contracting && spring_length > 0.05 {
                let f = ((render.edge_length / speed * second) * 10.0) / spring_length;
                render.lerp([v, u], f);
            } else {
                //let diff = render.positions[v] - render.positions[u];
                let target_length = diameter_spring_length * shape.diameter_percent([v, u]);
                let f = (target_length - spring_length) / speed * second;
                render.apply_scalar([v, u], f);
            }
        }
    }

    pub fn face_centroid(&self, face_index: usize) -> Vec3 {
        // All vertices associated with this face
        self.shape.cycles[face_index]
            .iter()
            .map(|&v| self.render.positions[v])
            .fold(Vec3::zero(), |a, b| a + b)
            / self.shape.cycles[face_index].len() as f32
    }

    /// Outward-pointing unit normal of a face, via Newell's method.
    /// Sign is corrected against the face centroid, since cycles have no winding-order guarantee.
    pub fn face_normal(&self, face_index: usize) -> Vec3 {
        let cycle = &self.shape.cycles[face_index];
        let n = cycle.len();
        let mut normal = Vec3::zero();
        for i in 0..n {
            let current = self.render.positions[cycle[i]];
            let next = self.render.positions[cycle[(i + 1) % n]];
            normal.x += (current.y - next.y) * (current.z + next.z);
            normal.y += (current.z - next.z) * (current.x + next.x);
            normal.z += (current.x - next.x) * (current.y + next.y);
        }
        let normal = normal.normalized();
        let centroid = self.face_centroid(face_index);
        if normal.dot(centroid) < 0.0 {
            -normal
        } else {
            normal
        }
    }

    /// Inscribed-circle radius of a face: the distance from its centroid to its nearest edge.
    fn face_inradius(&self, face_index: usize, centroid: Vec3) -> f32 {
        let cycle = &self.shape.cycles[face_index];
        let n = cycle.len();
        (0..n)
            .map(|i| {
                let a = self.render.positions[cycle[i]];
                let b = self.render.positions[cycle[(i + 1) % n]];
                let ab = b - a;
                let t = (centroid - a).dot(ab) / ab.mag_sq();
                (centroid - (a + ab * t.clamp(0.0, 1.0))).mag()
            })
            .fold(f32::MAX, f32::min)
    }

    /// Distance from a 2D polygon's local origin to its boundary along a unit direction.
    /// Used so containment is measured against face 0's true shape, not a circle approximation.
    fn polygon_boundary_distance(poly: &[(f32, f32)], dir: (f32, f32)) -> f32 {
        let n = poly.len();
        (0..n)
            .filter_map(|i| {
                let (ax, ay) = poly[i];
                let (bx, by) = poly[(i + 1) % n];
                let (ex, ey) = (bx - ax, by - ay);
                let d = ex * dir.1 - ey * dir.0;
                if d.abs() < 1e-9 {
                    return None;
                }
                let t = (ex * ay - ey * ax) / d;
                let s = (dir.0 * ay - dir.1 * ax) / d;
                (t > 0.0 && (-1e-3..=1.0 + 1e-3).contains(&s)).then_some(t)
            })
            .fold(f32::MAX, f32::min)
    }

    /// Largest eye_offset for which every vertex still projects inside face 0's true boundary.
    /// The depth epsilon is scaled to `inradius` to avoid flicker from simulation noise.
    pub fn schlegel_safe_eye_offset(&self, requested: f32) -> f32 {
        let centroid = self.face_centroid(0);
        let normal = self.face_normal(0);
        let reference = self.render.positions[self.shape.cycles[0][0]] - centroid;
        let u = reference.normalized();
        let v = normal.cross(u).normalized();
        let polygon: Vec<(f32, f32)> = self.shape.cycles[0]
            .iter()
            .map(|&i| {
                let q = self.render.positions[i] - centroid;
                (q.dot(u), q.dot(v))
            })
            .collect();

        let depth_epsilon = self.face_inradius(0, centroid) * 0.02;
        let bound = self.render.positions.iter().fold(f32::MAX, |bound, &p| {
            let q = p - centroid;
            let depth = -q.dot(normal); // distance behind face 0's plane; convex faces give depth >= 0
            let lateral = q - q.dot(normal) * normal;
            let lateral_mag = lateral.mag();
            if depth <= depth_epsilon || lateral_mag < 1e-6 {
                return bound;
            }
            let dir = (lateral.dot(u) / lateral_mag, lateral.dot(v) / lateral_mag);
            let boundary = Self::polygon_boundary_distance(&polygon, dir);
            if lateral_mag > boundary {
                bound.min(depth * boundary / (lateral_mag - boundary))
            } else {
                bound
            }
        });
        requested.min(bound * 0.9).clamp(0.02, requested)
    }

    /// Camera params for a Schlegel-diagram view through face 0 at a given eye_offset.
    /// Fitting the FOV to face 0's own circumradius alone is sufficient, given `schlegel_safe_eye_offset`.
    pub fn schlegel_camera_from_offset(&self, eye_offset: f32) -> (Vec3, Vec3, Vec3, f32, f32, f32) {
        let centroid = self.face_centroid(0);
        let normal = self.face_normal(0);
        let reference_vertex = self.render.positions[self.shape.cycles[0][0]];

        let eye = centroid + normal * eye_offset;
        let target = centroid - normal;
        let up = (reference_vertex - centroid).normalized();

        let circumradius = self.shape.cycles[0]
            .iter()
            .map(|&v| (self.render.positions[v] - centroid).mag())
            .fold(0.0_f32, f32::max);
        let half_fov = (circumradius / eye_offset).atan();
        let fov_y = (2.0 * half_fov / SCHLEGEL_FOV_FILL).clamp(0.2, std::f32::consts::PI - 0.1);

        let max_dist = self
            .render
            .positions
            .iter()
            .map(|&p| (p - eye).mag())
            .fold(0.0_f32, f32::max);
        let near = (eye_offset * 0.1).max(0.01);
        let far = (max_dist * 1.2).max(near + 1.0);

        (eye, target, up, fov_y, near, far)
    }

    pub fn moment_vertices(&self, colors: &[crate::render::color::RGBA]) -> Vec<MomentVertex> {
        let Polyhedron { shape, render, .. } = self;

        // Polygon side count -> color
        let color_map: HashMap<usize, Vec4> =
            shape.cycles.iter().fold(HashMap::new(), |mut acc, c| {
                if !acc.contains_key(&c.len()) {
                    acc.insert(c.len(), colors[acc.len() % colors.len()].into());
                }
                acc
            });

        shape
            .cycles
            .iter()
            .flat_map(|cycle| {
                let positions: Vec<Vec3> = match cycle.len() {
                    3 => cycle.iter().map(|&i| render.positions[i]).collect(),
                    4 => [0, 1, 2, 2, 3, 0]
                        .iter()
                        .map(move |&i| render.positions[cycle[i]])
                        .collect(),
                    _ => {
                        let centroid: Vec3 = cycle
                            .iter()
                            .map(|&c| render.positions[c])
                            .fold(Vec3::zero(), std::ops::Add::add)
                            / cycle.len() as f32;

                        (0..cycle.len())
                            .flat_map(move |i| {
                                vec![
                                    render.positions[cycle[i]],
                                    centroid,
                                    render.positions[cycle[i + 1]],
                                ]
                            })
                            .collect()
                    }
                };

                // Colors are determined by cycle length
                let color = color_map[&cycle.len()];
                // Map into MomentVertices
                positions
                    .into_iter()
                    .map(move |position| MomentVertex::new(position, color))
            })
            .collect()
    }

    // fn face_positions(&self, face_index: usize) -> Vec<Vec3> {
    //     self.shape.cycles[face_index]
    //         .iter()
    //         .map(|&v| self.render.vertices[v].position)
    //         .collect()
    // }
    // Use a Fibonacci Lattice to spread the points evenly around a sphere
    // pub fn connect(&mut self, [v, u]: [VertexId; 2]) {
    //     self.graph.connect([v, u]);
    // }
    //
    // pub fn disconnect(&mut self, [v, u]: [VertexId; 2]) {
    //     self.graph.disconnect([v, u]);
    // }
    //
    // pub fn insert(&mut self) -> VertexId {
    //     self.positions
    //         .push(Vec3::new(random(), random(), random()).normalized());
    //     self.speeds.push(Vec3::zero());
    //     self.graph.insert()
    // }

    // pub fn delete(&mut self, v: VertexId) {
    //     self.vertices.remove(&v);
    //
    //     self.edges = self
    //         .edges
    //         .clone()
    //         .into_iter()
    //         .filter(|e| !e.contains(v))
    //         .collect();
    //
    //     self.cycles = self
    //         .cycles
    //         .clone()
    //         .into_iter()
    //         .map(|face| face.into_iter().filter(|&u| u != v).collect())
    //         .collect();
    //
    //     self.positions.remove(&v);
    //     self.speeds.remove(&v);
    // }
    //
    // /// Edges of a vertex
    // pub fn edges(&self, v: VertexId) -> Vec<Edge> {
    //     let mut edges = vec![];
    //     for u in 0..self.dist.len() {
    //         if self.dist[v][u] == 1 {
    //             edges.push((v, u).into());
    //         }
    //     }
    //     edges
    // }

    // /// Number of faces
    // pub fn face_count(&self) -> i64 {
    //     2 + self.edges.len() as i64 - self.vertices.len() as i64
    // }

    //
    //
    //
}

// impl Display for PolyGraph {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let mut vertices = self.vertices.iter().collect::<Vec<_>>();
//         vertices.sort();
//         let mut adjacents = self.edges.clone().into_iter().collect::<Vec<_>>();
//         adjacents.sort();
//
//         f.write_fmt(format_args!(
//             "name:\t\t{}\nvertices:\t{:?}\nadjacents:\t{}\nfaces:\t\t{}\n",
//             self.name,
//             vertices,
//             adjacents
//                 .iter()
//                 .fold(String::new(), |acc, e| format!("{e}, {acc}")),
//             self.cycles.iter().fold(String::new(), |acc, f| format!(
//                 "[{}], {acc}",
//                 f.iter().fold(String::new(), |acc, x| format!("{x}, {acc}"))
//             ))
//         ))
//     }
// }
