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
    /// Returns each new vertex's originating vertex, so render can re-seed positions.
    pub fn expand(&mut self) -> Vec<VertexId> {
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
        for (f, cycle) in cycles.iter().enumerate() {
            let n = cycle.len();
            for k in 0..n {
                distance.connect([c[f][k], c[f][(k + 1) % n]]);
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
        parents
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
