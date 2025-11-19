mod conway;
mod cycles;
mod distance;
mod platonic;
use std::{fmt::Display, fs::File, io::Write, ops::Range};

use cycles::*;
use distance::*;

#[cfg(test)]
mod test;

use crate::polyhedron::*;

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
        File::create("./graph.svg")
            .unwrap()
            .write_all(&self.svg)
            .unwrap();
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

        // let mut edges = HashMap::default();
        let mut edges = Distance::new_max(self.distance.order());

        for &i in relevant.iter() {
            // Find the values that came before and after in the face
            let neighbors = self.cycles[i].neighbors(&v).unwrap();
            edges[neighbors] = i;
        }

        // println!("edges: {edges}");

        let mut true_edges: Vec<[VertexId; 2]> = edges
            .vertex_pairs()
            .filter_map(|p| {
                if edges[p] == usize::MAX {
                    None
                } else {
                    Some(p)
                }
            })
            .collect();

        // println!("true_edges: {true_edges:?}");

        let mut first = false;
        let mut face = vec![normalize_edge(true_edges[0])[0]];
        while !true_edges.is_empty() {
            let v = if first {
                *face.first().unwrap()
            } else {
                *face.last().unwrap()
            };
            if let Some(i) = true_edges.iter().position(|e| e.contains(&v)) {
                let next = if true_edges[i][0] == v {
                    true_edges[i][1]
                } else {
                    true_edges[i][0]
                };
                if !face.contains(&next) {
                    face.push(next);
                }
                true_edges.remove(i);
            } else {
                first ^= true;
            }
        }

        // println!("face: {face:?}");

        let mut ordered_face_indices = vec![];
        for i in 0..face.len() {
            let fi = edges[[face[i], face[(i + 1) % face.len()]]];
            ordered_face_indices.push(fi);
        }

        // println!("ordered: {ordered_face_indices:?}");

        ordered_face_indices
    }

    pub fn delete(&mut self, v: VertexId) {
        self.distance.delete(v);
        self.cycles.delete(v);
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
