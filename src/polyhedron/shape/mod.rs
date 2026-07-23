mod conway;
mod cycles;
mod distance;
mod platonic;
mod topology;
use std::{fmt::Display, ops::Range};

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
    /// Next never-yet-used face id, for genuinely new faces only.
    next_face_id: FaceId,
    /// Fresh face id mapped to the face it was carved from, consumed and cleared by the color finalize.
    pub birth_parents: std::collections::HashMap<FaceId, FaceId>,
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
        // Find and save cycles
        self.cycles = Cycles::discover(&self.distance, &mut self.next_face_id);
        self.recompute_metrics();
    }

    /// Recomputes distances and springs but not faces, for operations that build their cycles explicitly.
    pub fn recompute_metrics(&mut self) {
        // Update the distance matrix in place
        self.distance.bfs_apsp();
        // Find and save springs
        self.springs = self.distance.springs();
    }

    /// Mints the next never-yet-used face id.
    fn fresh_face_id(&mut self) -> FaceId {
        let id = self.next_face_id;
        self.next_face_id += 1;
        id
    }

    /// Installs an explicitly-built face list: canonically sort it and refresh derived metrics.
    /// Callers that maintain the discovery invariant should follow with `assert_cycles_match_discovery`.
    fn install_cycles(&mut self, cycles: Vec<Vec<VertexId>>, ids: Vec<FaceId>) {
        self.cycles = Cycles::new(cycles, ids);
        self.cycles.sort();
        self.recompute_metrics();
    }

    /// Debug oracle asserting operation-built cycles equal discovery's, as canonicalized faces in order.
    /// This replaces the self-healing that per-op rediscovery used to provide.
    pub fn assert_cycles_match_discovery(&self) {
        #[cfg(debug_assertions)]
        {
            let canonical = |cycles: &Cycles| -> Vec<Vec<VertexId>> {
                cycles
                    .iter()
                    .map(|c| {
                        let mut vs: Vec<VertexId> = c.iter().copied().collect();
                        vs.sort_unstable();
                        vs
                    })
                    .collect()
            };
            let mut scratch_id = 0;
            let discovered = Cycles::discover(&self.distance, &mut scratch_id);
            assert_eq!(
                canonical(&self.cycles),
                canonical(&discovered),
                "operation-built cycles diverge from discovery"
            );
        }
    }

    /// Edge removal still falls back to full rediscovery, so face ids and colors reset here.
    /// Only the unfinished Join operation uses it; switch to explicit cycle splicing when Join lands.
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
