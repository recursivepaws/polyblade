use ultraviolet::Vec3;

use super::{Cycle, Cycles, Shape};
use crate::polyhedron::{normalize_edge, shape::Distance, VertexId};
use std::collections::{HashMap, HashSet};

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

    /* pub fn expand(&mut self) {

        self.distance.exp
        // Delete a
        // for
        // for i in 0..self.cycles.len() {
        //     self.cycles[i].replace(v, u);
        // }
        self.recompute();
    } */

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
            let v = self.distance.insert();
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

    pub fn chamfer(&mut self) {
        let originals = self.edges().collect::<Vec<_>>();
        for cycle in self.cycles.iter() {
            let mut new_face = vec![];
            for &v in cycle.iter() {
                let u = self.distance.insert();
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

    pub fn medial(&mut self) {
        // Store original edges and create a mapping to new vertices
        let original_edges: Vec<[VertexId; 2]> = self.distance.edges().collect();
        let mut edge_to_new_vertex: HashMap<[VertexId; 2], VertexId> = HashMap::new();

        // Create a new vertex for each original edge
        for &edge in &original_edges {
            let new_v = self.distance.insert();
            edge_to_new_vertex.insert(edge, new_v);
            // Also store the reversed edge for lookup convenience
            edge_to_new_vertex.insert([edge[1], edge[0]], new_v);
        }

        // For each original face, connect the new edge-vertices around it
        // This creates the "face faces"
        for cycle in self.cycles.iter() {
            for i in 0..cycle.len() {
                let v = cycle[i];
                let u = cycle[i + 1]; // Cycle has wrap-around indexing
                let w = cycle[i + 2];

                // Get new vertices for consecutive edges around this face
                if let (Some(&nv1), Some(&nv2)) = (
                    edge_to_new_vertex.get(&[v, u]),
                    edge_to_new_vertex.get(&[u, w]),
                ) {
                    self.distance.connect([nv1, nv2]);
                }
            }
        }

        // For each original vertex, connect all incident edge-vertices
        // This creates the "vertex faces"
        let original_vertex_count = self.distance.order() - original_edges.len();
        for v in 0..original_vertex_count {
            // Get all edges incident to this vertex, in cyclic order
            let incident_edges = self.cycles.sorted_connections(v);

            // Connect the new vertices corresponding to consecutive incident edges
            for i in 0..incident_edges.len() {
                let edge1 = incident_edges[i];
                let edge2 = incident_edges[(i + 1) % incident_edges.len()];

                if let (Some(&nv1), Some(&nv2)) = (
                    edge_to_new_vertex.get(&[v, edge1]),
                    edge_to_new_vertex.get(&[v, edge2]),
                ) {
                    self.distance.connect([nv1, nv2]);
                }
            }
        }

        // Disconnect all original edges
        for &edge in &original_edges {
            self.distance.disconnect(edge);
        }

        // Recompute cycles from the new graph structure
        self.recompute();
    }

    pub fn dual(&mut self) {
        let original_vertex_count = self.distance.order();
        let original_edges: Vec<[VertexId; 2]> = self.distance.edges().collect();

        // Create a new vertex for each original face
        let mut face_to_vertex: HashMap<usize, VertexId> = HashMap::new();
        for (face_idx, _) in self.cycles.iter().enumerate() {
            let new_v = self.distance.insert();
            face_to_vertex.insert(face_idx, new_v);
        }

        // For each original edge, connect the two faces that shared it
        for edge in original_edges {
            let mut faces_with_edge = Vec::new();

            for (face_idx, cycle) in self.cycles.iter().enumerate() {
                // Check if this edge appears in the cycle
                // let has_edge = (0..cycle.len()).any(|i| {
                //     let v = cycle[i];
                //     let u = cycle[i + 1]; // Cycle has wrap-around indexing
                //     (v == edge[0] && u == edge[1]) || (v == edge[1] && u == edge[0])
                // });

                if cycle.contains_edge(edge) {
                    faces_with_edge.push(face_idx);
                }
            }

            // Each edge should be shared by exactly 2 faces in a valid polyhedron
            if faces_with_edge.len() == 2 {
                let v1 = face_to_vertex[&faces_with_edge[0]];
                let v2 = face_to_vertex[&faces_with_edge[1]];
                self.distance.connect([v1, v2]);
            }
        }

        // Delete all original vertices (in reverse order so indices don't shift incorrectly)
        for v in (0..original_vertex_count).rev() {
            self.distance.delete(v);
        }

        self.recompute();
    }

    pub fn _expand(&mut self, snub: bool) -> Vec<[VertexId; 2]> {
        // Helper to normalize edges (smaller vertex first)
        // let mut new_edges = HashSet::<[VertexId; 2]>::default();
        let mut new_edges = Distance::new_max(self.distance.order() * 10);
        let mut face_edges = Distance::new_max(self.distance.order() * 10);

        let ordered_face_indices: HashMap<usize, Vec<usize>> = self
            .distance
            .vertices()
            .map(|v| (v, self.ordered_face_indices(v)))
            .collect();

        println!("ordered face indices:\n {:#?}", ordered_face_indices);

        // For every vertex
        for v in self.distance.vertices() {
            // let original_position = self.positions[&v];
            // let original_position = Vec3::zero();
            //
            let mut new_face = Cycle::default();
            // For every face which contains the vertex
            for &i in ordered_face_indices.get(&v).unwrap() {
                // Create a new vertex
                let u = self.distance.insert();
                // Replace it in the face
                println!("replacing {v} with {u} in {:#?}", self.cycles[i]);
                self.cycles[i].replace(v, u);
                // Now replace
                // let ui = self.cycles[i].iter().position(|&x| x == u).unwrap();
                // let flen = self.cycles[i].len();
                // Find the values that came before and after in the face
                // let a = self.cycles[i][(ui + flen - 1) % flen];
                // let b = self.cycles[i][(ui + 1) % flen];
                if let Some([a, b]) = self.cycles[i].neighbors(&u) {
                    // Remove existing edges which may no longer be accurate
                    new_edges.disconnect([a, v]);
                    new_edges.disconnect([b, v]);
                    // Add the new edges which are so yass
                    new_edges.connect([a, u]);
                    new_edges.connect([b, u]);
                    // Add u to the new face being formed
                    new_face.push(u);
                }
                // pos
                // self.positions.insert(u, original_position);
            }
            for i in 0..new_face.len() {
                face_edges.connect([new_face[i], new_face[i + 1]]);
            }

            self.cycles.push(new_face);
            self.delete(v);
        }

        let mut solved_edges = Distance::new_max(self.distance.order() * 10);

        // For every triangle / nf edge
        for a in face_edges.clone().edges() {
            // find the edge which is parallel to it
            for b in face_edges.edges() {
                let [av, au] = a;
                let [bv, bu] = b;

                if !solved_edges.connected(a) && !solved_edges.connected(b) {
                    if new_edges.connected([av, bv]) && new_edges.connected([au, bu]) {
                        let quad = Cycle::from(vec![bu, au, av, bv]);
                        self.cycles.push(quad);

                        solved_edges.connect(a);
                        solved_edges.connect(b);
                    }

                    if new_edges.connected([au, bv]) && new_edges.connected([av, bu]) {
                        if snub {
                            new_edges.connect([au, bu]);
                            let m = Cycle::from(vec![au, bu, av]);
                            let n = Cycle::from(vec![au, bu, bv]);
                            self.cycles.push(m);
                            self.cycles.push(n);
                        } else {
                            let quad = Cycle::from(vec![au, bv, bu, av]);
                            self.cycles.push(quad);
                        }
                        solved_edges.connect(a);
                        solved_edges.connect(b);
                    }
                }
            }
        }
        // self.edges = HashSet::default();
        // self.edges.extend(new_edges.clone());
        // self.edges.extend(face_edges);
        let connectme = [
            new_edges.edges().collect::<Vec<_>>(),
            face_edges.edges().collect(),
        ]
        .concat();

        println!("connectme: {:#?}", connectme);
        println!("order: {:#?}", self.distance.order());
        self.distance.insert();

        for edge in connectme {
            self.distance.connect(edge);
        }

        self.recompute();

        new_edges.edges().collect()
    }

    pub fn expand(&mut self) -> Vec<[VertexId; 2]> {
        let ordered_face_indices: HashMap<usize, Vec<usize>> = self
            .distance
            .vertices()
            .map(|v| (v, self.ordered_face_indices(v)))
            .collect();

        let mut count = 0;
        for v in self.distance.vertices() {
            // let existing = ordered_face_indices[&v].clone();
            //
            println!("distance: {}", self.distance);
            println!("cycles: {:?}", self.cycles);
            println!("obtaining ordered face indices of: {v}");

            let existing = self.ordered_face_indices(v);
            println!("existing: {existing:?}");

            // Construct new face object for reference
            let mut new_face = vec![v];
            // Insert n-1 new verticies
            for i in 1..existing.len() {
                let q = self.distance.insert();
                new_face.push(q);
                self.distance.disconnect([existing[i], v]);
                self.distance.connect([existing[i], q]);
            }

            println!("new: {new_face:?}");

            let cycle = Cycle::from(new_face);
            for v in cycle.iter() {
                // let [x, y] = cycle.neighbors(v).unwrap();
                // cycle[v + 1]
                // self.distance.connect([x, *v]);
                // self.distance.connect([y, *v]);
                self.distance.connect([*v, cycle[v + 1]]);
                self.distance.connect([*v, cycle[v + cycle.len() - 1]]);
            }

            self.recompute();
            count += 1;
            if count > 2 {
                break;
            }
        }

        self.recompute();

        return vec![];
    }
}
