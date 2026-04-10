use rand::random;
use ultraviolet::{Lerp as _, Vec3};

use super::{VertexId, SPEED_DAMPENING};

#[derive(Debug, Clone)]
pub struct Render {
    /// Positions in 3D space
    pub positions: Vec<Vec3>,
    /// Speeds in 3D space
    pub speeds: Vec<Vec3>,
    /// Edge length
    pub edge_length: f32,
}

//impl rand::
pub fn random_positions(n: usize) -> Vec<Vec3> {
    (0..n)
        .map(|_| Vec3::new(random(), random(), random()).normalized())
        .collect()
}

impl Render {
    pub fn new(n: usize) -> Self {
        Self {
            positions: random_positions(n),
            speeds: vec![Vec3::zero(); n],
            edge_length: 1.0,
        }
    }

    pub fn update(&mut self, speed: f32, second: f32) {
        self.center();
        self.resize(speed, second);
    }

    fn center(&mut self) {
        let shift =
            self.positions.iter().fold(Vec3::zero(), |a, &b| a + b) / self.positions.len() as f32;
        log::debug!("shifting all positions by {shift:?}");

        for p in self.positions.iter_mut() {
            *p -= shift;
        }
    }

    fn resize(&mut self, speed: f32, second: f32) {
        let mean_length = self.positions.iter().map(Vec3::mag).fold(0.0, f32::max);
        let matrixance = mean_length - 1.0;
        self.edge_length -= matrixance / speed * second;
    }

    pub fn new_capacity(&mut self, n: usize) {
        if n > self.positions.len() {
            let difference = n - self.positions.len();
            self.positions.extend(random_positions(difference));
            self.speeds.extend(vec![Vec3::zero(); difference]);
        }
    }

    pub fn extend(&mut self, n: usize, position: Vec3) {
        self.positions.extend(vec![position; n]);
        self.speeds.extend(vec![Vec3::zero(); n]);
    }

    pub fn spring_length(&self, [v, u]: [VertexId; 2]) -> f32 {
        (self.positions[v] - self.positions[u]).mag()
    }

    pub fn lerp(&mut self, [v, u]: [VertexId; 2], f: f32) {
        self.positions[v] = self.positions[v].lerp(self.positions[u], f);
        self.positions[u] = self.positions[u].lerp(self.positions[v], f);
    }

    // pub fn lattice(&mut self) {
    //     self.vertices = vec![];
    //     // Use a Fibonacci Lattice to evently distribute starting points on a sphere
    //     let phi = std::f32::consts::PI * (3.0 - 5.0f32.sqrt());
    //     for v in 0..self.graph.len() {
    //         let y = 1.0 - (v as f32 / (self.graph.len() - 1) as f32);
    //         let radius = (1.0 - y * y).sqrt();
    //         let theta = (phi * (v as f32)) % (std::f32::consts::PI * 2.0);
    //         let x = theta.cos() * radius;
    //         let z = theta.sin() * radius;
    //         self.positions.push(Vec3::new(x, y, z));
    //     }
    // }
    //

    pub fn apply_scalar(&mut self, [v, u]: [VertexId; 2], scalar: f32) {
        let diff = self.positions[v] - self.positions[u];
        let delta = diff * scalar;
        self.speeds[v] = (self.speeds[v] + delta) * SPEED_DAMPENING;
        self.speeds[u] = (self.speeds[u] - delta) * SPEED_DAMPENING;
        self.positions[v] += self.speeds[v];
        self.positions[u] += self.speeds[u];
    }

    pub fn contract_edges(&mut self, mut edges: Vec<[VertexId; 2]>) {
        // let mut transformed = HashSet::default();
        while !edges.is_empty() {
            // Pop an edge
            let [w, x] = edges.remove(0);
            let v = w.max(x);
            let _u = w.min(x);
            // if transformed.contains(&v) && transformed.contains(&u) {}

            self.positions.remove(v);
            self.speeds.remove(v);
            // transformed.insert(v);

            for [x, w] in &mut edges {
                if *x > v {
                    *x -= 1;
                }
                if *w > v {
                    *w -= 1;
                }
            }
        }
    }
}
