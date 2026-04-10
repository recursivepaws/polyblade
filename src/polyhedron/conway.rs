use super::shape::cycles::*;
use super::shape::edge::*;
use std::collections::HashMap;

use crate::polyhedron::{Polyhedron, VertexId};

impl Polyhedron {
    pub fn split_vertex(&mut self, v: usize) -> Vec<[usize; 2]> {
        let Polyhedron { shape, render, .. } = self;
        let edges = shape.split_vertex(v);
        render.extend(edges.len() - 1, render.positions[v]);
        edges
    }

    pub fn truncate(&mut self, d: usize) -> Vec<[VertexId; 2]> {
        let mut new_edges = Vec::default();
        let vertices = self.shape.vertices().collect::<Vec<_>>();
        for v in vertices.into_iter().rev() {
            if d == 0 || self.shape.degree(v) == d {
                new_edges.extend(self.split_vertex(v));
                self.shape.recompute();
            }
        }
        new_edges
    }

    /// `a` ambo
    /// Returns a set of edges to contract
    pub fn ambo(&mut self) -> Vec<[VertexId; 2]> {
        // Truncate
        let new_edges = self.truncate(0);
        // [VertexId; 2]s that were already there get contracted
        self.shape
            .edges()
            .filter(|&[v, u]| !new_edges.contains(&[v, u]) && !new_edges.contains(&[u, v]))
            .collect()
    }

    pub fn contract(&mut self, edges: Vec<[VertexId; 2]>) {
        self.shape.contract_edges(edges.clone());
        self.render.contract_edges(edges);
    }

    pub fn ambo_contract(&mut self) {
        let edges = self.ambo();
        self.contract(edges);
        log::info!(
            "p: {}, d: {}",
            self.render.positions.len(),
            self.shape.order()
        );
    }

    // pub fn expand(&mut self) -> Vec<[VertexId; 2]> {
    //     vec![]
    // }
    /// `e` = `aa`
    pub fn expand(&mut self, snub: bool) -> EdgeSet {
        let mut new_edges = EdgeSet::default();
        let mut face_edges = EdgeSet::default();

        let ordered_face_indices: HashMap<VertexId, Vec<VertexId>> = self
            .shape
            .vertices()
            .into_iter()
            .map(|v| (v, self.shape.ordered_face_indices(v)))
            .collect();

        // For every vertex
        for v in self.shape.vertices().into_iter() {
            let original_position = self.render.positions[v];
            let mut new_face = Cycle::default();
            // For every face which contains the vertex
            for &i in ordered_face_indices.get(&v).unwrap() {
                // Create a new vertex
                let u = self.shape.insert();
                // Replace it in the face
                self.shape.cycles[i].replace(v, u);
                // Now replace
                let ui = self.shape.cycles[i].iter().position(|&x| x == u).unwrap();
                let flen = self.shape.cycles[i].len();
                // Find the values that came before and after in the face
                let a = self.shape.cycles[i][(ui + flen - 1) % flen];
                let b = self.shape.cycles[i][(ui + 1) % flen];
                // Remove existing edges which may no longer be accurate
                new_edges.remove([a, v]);
                new_edges.remove([b, v]);
                // Add the new edges which are so yass
                new_edges.insert([a, u]);
                new_edges.insert([b, u]);
                // Add u to the new face being formed
                new_face.push(u);
                // pos
                self.render.positions.insert(u, original_position);
            }
            for i in 0..new_face.len() {
                face_edges.insert([new_face[i], new_face[(i + 1) % new_face.len()]]);
            }
            self.shape.cycles.push(new_face);
            self.shape.delete(v);
        }

        let mut solved_edges = EdgeSet::default();

        // For every triangle / nf edge
        for a in face_edges.iter() {
            // find the edge which is parallel to it
            for b in face_edges.iter() {
                if !solved_edges.contains(a) && !solved_edges.contains(b) {
                    if new_edges.contains([a[0], b[0]]) && new_edges.contains([a[1], b[1]]) {
                        if snub {
                            new_edges.insert([a[0], b[1]]);
                            let m = Cycle::from(vec![a[0], b[1], a[1]]);
                            let n = Cycle::from(vec![a[0], b[1], b[0]]);
                            self.shape.cycles.push(m);
                            self.shape.cycles.push(n);
                        } else {
                            let quad = Cycle::from(vec![b[1], a[1], a[0], b[0]]);
                            self.shape.cycles.push(quad);
                        }

                        solved_edges.insert(a);
                        solved_edges.insert(b);
                    }

                    if new_edges.contains([a[1], b[0]]) && new_edges.contains([a[0], b[1]]) {
                        if snub {
                            new_edges.insert([a[1], b[1]]);
                            let m = Cycle::from(vec![a[1], b[1], a[0]]);
                            let n = Cycle::from(vec![a[1], b[1], b[0]]);
                            self.shape.cycles.push(m);
                            self.shape.cycles.push(n);
                        } else {
                            let quad = Cycle::from(vec![a[1], b[0], b[1], a[0]]);
                            self.shape.cycles.push(quad);
                        }
                        solved_edges.insert(a);
                        solved_edges.insert(b);
                    }
                }
            }
        }

        for e in new_edges.iter() {
            self.shape.connect(e);
        }
        for e in face_edges.iter() {
            self.shape.connect(e);
        }

        self.shape.recompute();
        // self.edges = HashSet::default();
        // self.edges.extend(new_edges.clone());
        // self.edges.extend(face_edges);
        // new_edges
        new_edges
    }

    pub fn chamfer(&mut self) {
        self.shape.chamfer();
    }
}
