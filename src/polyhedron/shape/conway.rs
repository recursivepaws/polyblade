use super::{Cycle, Cycles, Distance, Shape};
use crate::polyhedron::VertexId;
use std::collections::HashMap;

impl Shape {
    pub fn split_vertex(&mut self, v: VertexId) -> Vec<[usize; 2]> {
        let sc = self.cycles.sorted_connections(v);
        let edges = self.distance.split_vertex(v, sc);
        self.cycles = Cycles::from(&self.distance);
        edges
    }

    /// `t` full truncation: one new vertex per (vertex, incident-edge) corner.
    /// Returns the vertex-figure edges (so `ambo` contracts the rest) and each
    /// new vertex's originating vertex (so render can re-seed positions).
    pub fn truncate(&mut self) -> (Vec<[VertexId; 2]>, Vec<VertexId>) {
        // Index every (vertex, neighbor) corner; `corner[(v, u)]` is the new vertex there.
        let mut corner: HashMap<(VertexId, VertexId), VertexId> = HashMap::new();
        let mut parents: Vec<VertexId> = Vec::new();
        let mut vertex_order: Vec<Vec<VertexId>> = Vec::with_capacity(self.order());
        for v in self.vertices() {
            let sc = self.cycles.sorted_connections(v);
            for &u in &sc {
                corner.insert((v, u), parents.len());
                parents.push(v);
            }
            vertex_order.push(sc);
        }

        let mut distance = Distance::new(parents.len());
        // Vertex-figure d-gon: link a vertex's corners in cyclic neighbor order.
        // The truncated faces (2n-gons) emerge for free from these plus the original edges.
        let mut new_edges = Vec::new();
        for (v, sc) in vertex_order.iter().enumerate() {
            let d = sc.len();
            for i in 0..d {
                let edge = [corner[&(v, sc[i])], corner[&(v, sc[(i + 1) % d])]];
                distance.connect(edge);
                new_edges.push(edge);
            }
        }
        // Original edges: each keeps its two endpoint corners joined.
        for [v, u] in self.edges() {
            distance.connect([corner[&(v, u)], corner[&(u, v)]]);
        }

        distance.inherit_ancestry(&self.distance, &parents);
        self.distance = distance;
        self.recompute();
        (new_edges, parents)
    }

    pub fn contract_edges(&mut self, edges: Vec<[VertexId; 2]>) {
        self.distance.contract_edges(edges);
        // Delete a
        // for
        // for i in 0..self.cycles.len() {
        //     self.cycles[i].replace(v, u);
        // }
        self.recompute();
    }

    pub fn kis(&mut self, degree: Option<usize>) -> Vec<[VertexId; 2]> {
        let edges = self.distance.edges().collect();
        let cycles: Vec<&Cycle> = self
            .cycles
            .iter()
            .filter(move |cycle| {
                if let Some(degree) = degree {
                    cycle.len() == degree
                } else {
                    true
                }
            })
            .collect();

        for cycle in cycles {
            let parents: Vec<VertexId> = cycle.iter().copied().collect();
            let v = self.distance.insert_from(&parents);
            // let mut vpos = Vec3::zero();

            for &u in cycle.iter() {
                self.distance.connect([v, u]);
                //vpos += self.positions[&u];
            }

            //self.positions.insert(v, vpos / cycle.len() as f32);
        }

        self.recompute();
        edges
    }

    /// `e` expand (cantellation): one new vertex per original vertex-face corner.
    /// Returns each new vertex's originating vertex (so render can re-seed positions)
    /// and the face-figure edges (contracting them collapses each face to a point,
    /// yielding the dual).
    pub fn expand(&mut self) -> (Vec<VertexId>, Vec<[VertexId; 2]>) {
        let cycles: Vec<Vec<VertexId>> = self
            .cycles
            .iter()
            .map(|c| c.iter().copied().collect())
            .collect();

        // Index every (face, corner) incidence; `c[f][i]` is the new vertex there.
        let mut c: Vec<Vec<VertexId>> = Vec::with_capacity(cycles.len());
        let mut parents: Vec<VertexId> = Vec::new();
        for cycle in &cycles {
            let row = cycle
                .iter()
                .map(|&v| {
                    parents.push(v);
                    parents.len() - 1
                })
                .collect();
            c.push(row);
        }

        // Which two faces each original edge borders.
        let mut edge_faces: HashMap<[VertexId; 2], Vec<usize>> = HashMap::new();
        for (f, cycle) in cycles.iter().enumerate() {
            let n = cycle.len();
            for k in 0..n {
                let (a, b) = (cycle[k], cycle[(k + 1) % n]);
                let edge = if a < b { [a, b] } else { [b, a] };
                edge_faces.entry(edge).or_default().push(f);
            }
        }

        let mut distance = Distance::new(parents.len());
        // Face-figure edges: the original n-gon, using this face's corner copies.
        // Contracting these collapses each face to a point, giving the dual.
        let mut face_edges = Vec::new();
        for (f, cycle) in cycles.iter().enumerate() {
            let n = cycle.len();
            for k in 0..n {
                let edge = [c[f][k], c[f][(k + 1) % n]];
                distance.connect(edge);
                face_edges.push(edge);
            }
        }
        // Vertex-figure rungs: link the two faces' copies of each endpoint.
        // The edge quads emerge for free as chordless 4-cycles of ff-edges + rungs.
        for (edge, faces) in &edge_faces {
            if faces.len() != 2 {
                continue;
            }
            let [f, g] = [faces[0], faces[1]];
            for &v in edge {
                let pf = cycles[f].iter().position(|&x| x == v).unwrap();
                let pg = cycles[g].iter().position(|&x| x == v).unwrap();
                distance.connect([c[f][pf], c[g][pg]]);
            }
        }

        distance.inherit_ancestry(&self.distance, &parents);
        self.distance = distance;
        self.recompute();
        (parents, face_edges)
    }

    pub fn chamfer(&mut self) {
        let originals = self.edges().collect::<Vec<_>>();
        for cycle in self.cycles.iter() {
            let mut new_face = vec![];
            for &v in cycle.iter() {
                let u = self.distance.insert_from(&[v]);
                new_face.push(u);
                self.distance.connect([v, u]);
            }
            for i in 0..new_face.len() {
                self.distance
                    .connect([new_face[i], new_face[(i + 1) % new_face.len()]]);
            }
        }
        for edge in originals {
            self.distance.disconnect(edge);
        }
        self.recompute();
    }
}
