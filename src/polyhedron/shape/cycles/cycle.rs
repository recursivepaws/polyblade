use crate::polyhedron::VertexId;
use std::ops::{Index, IndexMut};

#[derive(Default, Debug, Clone)]
pub struct Cycle(pub(super) Vec<VertexId>);

impl Index<usize> for Cycle {
    type Output = VertexId;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index.rem_euclid(self.0.len())]
    }
}

impl IndexMut<usize> for Cycle {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let i = index.rem_euclid(self.0.len());
        &mut self.0[i]
    }
}

impl Cycle {
    pub fn from(vertices: Vec<VertexId>) -> Self {
        Self(vertices)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Merges deleted `v` into survivor `u < v`, shifting higher indices down and collapsing consecutive duplicates.
    /// Returns whether the face survives with at least 3 vertices.
    pub fn contract_vertex(&mut self, v: VertexId, u: VertexId) -> bool {
        debug_assert!(u < v, "survivor must be the lower index");
        let mut out: Vec<VertexId> = Vec::with_capacity(self.0.len());
        for &x in &self.0 {
            let x = match x {
                x if x == v => u,
                x if x > v => x - 1,
                x => x,
            };
            if out.last() != Some(&x) {
                out.push(x);
            }
        }
        while out.len() > 1 && out.first() == out.last() {
            out.pop();
        }
        // A non-consecutive duplicate means the contract set pinched a face, which no operation should produce.
        debug_assert!(
            out.len() < 3
                || out.iter().collect::<std::collections::HashSet<_>>().len() == out.len(),
            "contraction pinched a face: {out:?}"
        );
        self.0 = out;
        self.0.len() >= 3
    }

    pub fn iter(&self) -> std::slice::Iter<'_, usize> {
        self.0.iter()
    }

    #[allow(dead_code)]
    pub fn contains(&self, v: &VertexId) -> bool {
        self.0.contains(v)
    }

    #[allow(dead_code)]
    pub fn push(&mut self, v: VertexId) {
        self.0.push(v);
    }
}
