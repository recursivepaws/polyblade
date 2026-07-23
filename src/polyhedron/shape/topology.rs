use super::Cycles;
use crate::polyhedron::{FaceId, VertexId};
use std::collections::{HashMap, HashSet};

/// Order-independent key for an undirected edge.
/// TODO: maybe we should rewrite this as a custom type
pub(super) fn undirected(a: VertexId, b: VertexId) -> [VertexId; 2] {
    if a < b { [a, b] } else { [b, a] }
}

/// Read-only snapshot of a shape's faces taken at the start of a Conway operation
pub(super) struct FaceTopology {
    /// The original faces
    pub(super) cycles: Vec<Vec<VertexId>>,
    /// The IDs of those original faces
    pub(super) ids: Vec<FaceId>,
    /// Vertex `v` in face `f` = pos[f][&v]
    pub(super) pos: Vec<HashMap<VertexId, usize>>,
    /// Undirected original edge bordering face indices
    edge_faces: HashMap<[VertexId; 2], Vec<usize>>,
}

impl FaceTopology {
    pub(super) fn snapshot(cycles: &Cycles) -> Self {
        let ids = cycles.ids().to_vec();
        let cycles: Vec<Vec<VertexId>> =
            cycles.iter().map(|c| c.iter().copied().collect()).collect();
        let mut pos: Vec<HashMap<VertexId, usize>> = Vec::with_capacity(cycles.len());
        let mut edge_faces: HashMap<[VertexId; 2], Vec<usize>> = HashMap::new();
        for (f, cycle) in cycles.iter().enumerate() {
            let n = cycle.len();
            let mut row = HashMap::with_capacity(n);
            for k in 0..n {
                row.insert(cycle[k], k);
                edge_faces
                    .entry(undirected(cycle[k], cycle[(k + 1) % n]))
                    .or_default()
                    .push(f);
            }
            pos.push(row);
        }
        Self {
            cycles,
            ids,
            pos,
            edge_faces,
        }
    }

    /// Index of vertex `v` within face `f`.
    pub(super) fn pos(&self, f: usize, v: VertexId) -> usize {
        self.pos[f][&v]
    }

    /// The face across `edge` from `f`, if `edge` is interior aka borders exactly two faces.
    pub(super) fn other_face(&self, f: usize, a: VertexId, b: VertexId) -> Option<usize> {
        let faces = self.edge_faces.get(&undirected(a, b))?;
        (faces.len() == 2).then(|| if faces[0] == f { faces[1] } else { faces[0] })
    }

    /// Visits each interior original edge exactly once in face order then corner order.
    /// Yields the face `f` it was found in, its endpoints `a,b` in `f`'s winding, and the opposite face `g`.
    pub(super) fn for_each_interior_edge(
        &self,
        mut visit: impl FnMut(usize, VertexId, VertexId, usize),
    ) {
        let mut seen: HashSet<[VertexId; 2]> = HashSet::new();
        // For each face
        for (f, cycle) in self.cycles.iter().enumerate() {
            let n = cycle.len();
            for k in 0..n {
                let (a, b) = (cycle[k], cycle[(k + 1) % n]);
                if seen.insert(undirected(a, b))
                    && let Some(g) = self.other_face(f, a, b)
                {
                    visit(f, a, b, g);
                }
            }
        }
    }
}
