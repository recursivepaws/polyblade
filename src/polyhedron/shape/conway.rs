use ultraviolet::Vec3;

use super::{Cycle, Cycles, Shape};
use crate::polyhedron::{normalize_edge, shape::Distance, VertexId};
use std::collections::{HashMap, HashSet};

impl Shape {
    pub fn split_vertex(&mut self, v: VertexId) -> Vec<[usize; 2]> {
        let connections = self.cycles.sorted_connections(v);
        log::info!("sorted connections for {v}: {connections:?}");
        // let edges = self.distance.split_vertex(v, sc.clone());

        // Remove the vertex
        let new_cycle: Cycle = Cycle::from(
            vec![v]
                .into_iter()
                .chain((1..connections.len()).map(|_| self.insert(Some(v))))
                .collect(),
        );

        for c in &connections {
            self.distance.disconnect([v, *c]);
        }

        for i in 0..new_cycle.len() {
            self.distance.connect([new_cycle[i], connections[i]]);
        }

        // track the edges that will compose the new face
        let mut new_edges = vec![];
        for i in 0..new_cycle.len() {
            let edge = [new_cycle[i], new_cycle[i + 1]];
            self.distance.connect(edge);
            new_edges.push(edge);
        }

        // new_edges
        //
        // for connection in sc {
        //     self.set_position(v, connection);
        // }
        self.cycles = Cycles::from(&self.distance);
        new_edges
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

    /* pub fn ofi(&self) {
        let ordered_face_indices: std::collections::HashMap<usize, Vec<usize>> = self
            .distance
            .vertices()
            .map(|v| (v, self.ordered_vertices(v)))
            .collect();

        log::info!(
            "incident vertices arranged in cyclic order: {:?}",
            ordered_face_indices
        );
    } */

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

    pub fn expand(&mut self) {
        log::info!("cycles: {:?}", self.cycles);

        let original_edges: Vec<[VertexId; 2]> = self.distance.edges().collect();

        // Cache incident edges for all vertices before modifying the graph
        let all_incident_edges: Vec<Vec<[VertexId; 2]>> =
            self.vertices().map(|i| self.incident_edges(i)).collect();

        // Disconnect all original edges
        for &edge in &original_edges {
            self.distance.disconnect(edge);
        }

        // Map each (vertex, neighbor) pair to a vertex ID in the expanded graph
        let mut edge_vertex_map: HashMap<(VertexId, VertexId), VertexId> = HashMap::new();

        // Step 1: Create vertex polygons
        for v in self.vertices() {
            let incident = &all_incident_edges[v];

            // First incident edge uses the original vertex
            let first_neighbor = incident[0][1];
            edge_vertex_map.insert((v, first_neighbor), v);

            // Remaining incident edges get new vertices
            for i in 1..incident.len() {
                let neighbor = incident[i][1];
                let new_v = self.insert(Some(neighbor));
                edge_vertex_map.insert((v, neighbor), new_v);
            }

            // Connect the vertices around this polygon
            for i in 0..incident.len() {
                let curr_neighbor = incident[i][1];
                let next_neighbor = incident[(i + 1) % incident.len()][1];

                let curr_vertex = edge_vertex_map[&(v, curr_neighbor)];
                let next_vertex = edge_vertex_map[&(v, next_neighbor)];

                self.distance.connect([curr_vertex, next_vertex]);
            }
        }

        // Step 2: Add cross-connections
        // for &[v, u] in &original_edges {
        //     let incident_v = &all_incident_edges[v];
        //     let incident_u = &all_incident_edges[u];
        //
        //     let curr_idx_v = incident_v.iter().position(|e| e[1] == u).unwrap();
        //     let next_idx_v = (curr_idx_v + 1) % incident_v.len();
        //
        //     let curr_idx_u = incident_u.iter().position(|e| e[1] == v).unwrap();
        //     let next_idx_u = (curr_idx_u + 1) % incident_u.len();
        //
        //     let curr_v = edge_vertex_map[&(v, incident_v[curr_idx_v][1])];
        //     let next_v = edge_vertex_map[&(v, incident_v[next_idx_v][1])];
        //
        //     let curr_u = edge_vertex_map[&(u, incident_u[curr_idx_u][1])];
        //     let next_u = edge_vertex_map[&(u, incident_u[next_idx_u][1])];
        //
        //     // Add both cross-connections (one will re-add the original edge)
        //     self.distance.connect([next_v, curr_u]);
        //     self.distance.connect([next_u, curr_v]);
        // }
        // Step 2: Add ALL cross-connections (this will re-add the needed original edges)
        for &[v, u] in &original_edges {
            let incident_v = &all_incident_edges[v];
            let incident_u = &all_incident_edges[u];

            let curr_idx_v = incident_v.iter().position(|e| e[1] == u).unwrap();
            let prev_idx_v = (curr_idx_v + incident_v.len() - 1) % incident_v.len(); // PREVIOUS, not next

            let curr_idx_u = incident_u.iter().position(|e| e[1] == v).unwrap();
            let prev_idx_u = (curr_idx_u + incident_u.len() - 1) % incident_u.len(); // PREVIOUS, not next

            let curr_v = edge_vertex_map[&(v, incident_v[curr_idx_v][1])];
            let prev_v = edge_vertex_map[&(v, incident_v[prev_idx_v][1])]; // prev instead of next

            let curr_u = edge_vertex_map[&(u, incident_u[curr_idx_u][1])];
            let prev_u = edge_vertex_map[&(u, incident_u[prev_idx_u][1])]; // prev instead of next

            // The square is: curr_v → curr_u → prev_u → prev_v → curr_v
            self.distance.connect([prev_v, curr_u]);
            self.distance.connect([curr_v, prev_u]);
        }

        self.recompute();
    }
}
