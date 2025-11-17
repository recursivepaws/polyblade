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
        for v in self.shape.vertices().rev() {
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
        // Edges that were already there get contracted
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

    pub fn chamfer(&mut self) {
        self.shape.chamfer();
    }
}
