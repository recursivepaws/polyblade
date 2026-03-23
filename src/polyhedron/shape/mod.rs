mod conway;
pub mod cycles;
mod distance;
pub mod edge;
mod platonic;
use std::{collections::HashSet, fmt::Display, ops::Range};

use cycles::*;
use distance::*;

#[cfg(test)]
mod test;

use crate::polyhedron::{shape::edge::EdgeMap, *};

/// Contains all properties that need to be computed iff the structure of the graph changes
#[derive(Default, Clone)]
pub(super) struct Shape {
    /// Graph, represented as Distance matrix
    distance: Distance,
    /// Cycles in the graph
    pub cycles: Cycles,
    /// Faces / chordless cycles
    pub springs: Vec<[VertexId; 2]>,
    /// SVG string of graph representation
    pub svg: Vec<u8>,
}

impl PartialEq for Shape {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl std::fmt::Debug for Shape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.distance.to_string())
    }
}

impl Display for Shape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.distance.to_string())
    }
}

impl Shape {
    pub fn order(&self) -> usize {
        self.distance.order()
    }

    pub fn insert(&mut self) -> usize {
        self.distance.insert()
    }

    pub fn delete(&mut self, v: usize) {
        self.distance.delete(v)
    }

    pub fn connect(&mut self, [v, u]: [VertexId; 2]) {
        self.distance.connect([v, u]);
    }

    pub fn degree(&self, v: usize) -> usize {
        self.distance.neighbors(v).len()
    }

    pub fn edges(&self) -> impl Iterator<Item = [VertexId; 2]> + use<'_> {
        self.distance.edges()
    }

    pub fn vertices(&self) -> Range<VertexId> {
        self.distance.vertices()
    }

    pub fn recompute(&mut self) {
        // Update the distance matrix in place
        self.distance.bfs_apsp();
        // Find and save cycles
        self.cycles = Cycles::from(&self.distance);
        // Find and save springs
        self.springs = self.distance.springs();
    }

    pub fn compute_graph_svg(&mut self) {
        self.svg = self.distance.svg().unwrap_or_default();
    }

    pub fn release(&mut self, edges: &[[VertexId; 2]]) {
        for &edge in edges {
            self.distance.disconnect(edge);
        }
        self.recompute();
    }

    /// Given a vertex pairing, what is their distance in G divided by the diameter of G
    pub fn diameter_percent(&self, [v, u]: [VertexId; 2]) -> f32 {
        self.distance[[v, u]] as f32 / self.distance.diameter() as f32
    }

    pub fn ordered_face_indices(&self, v: VertexId) -> Vec<usize> {
        let relevant = (0..self.cycles.len())
            .filter(|&i| self.cycles[i].contains(&v))
            .collect::<Vec<usize>>();

        let mut edges = EdgeMap::<usize>::default();

        for &i in relevant.iter() {
            let ui = self.cycles[i].iter().position(|&x| x == v).unwrap();
            let flen = self.cycles[i].len();
            // Find the values that came before and after in the face
            let a = self.cycles[i][(ui + flen - 1) % flen];
            let b = self.cycles[i][(ui + 1) % flen];

            edges.insert([a, b], i);
        }

        let f: Cycle = edges.keys().cloned().collect::<Vec<[VertexId; 2]>>().into();

        let mut ordered_face_indices = vec![];
        for i in 0..f.len() {
            let e: [VertexId; 2] = [f[i], f[(i + 1) % f.len()]];
            let fi = edges.get(e).unwrap();
            ordered_face_indices.push(*fi);
        }

        ordered_face_indices
    }
}

impl From<Distance> for Shape {
    fn from(distance: Distance) -> Self {
        let mut shape = Shape {
            distance,
            ..Default::default()
        };
        shape.recompute();
        shape
    }
}
