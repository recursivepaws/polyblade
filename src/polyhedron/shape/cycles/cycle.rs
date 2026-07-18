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

    #[allow(dead_code)]
    pub fn delete(&mut self, v: VertexId) {
        self.0 = self
            .0
            .clone()
            .into_iter()
            .filter_map(|u| {
                use std::cmp::Ordering::*;
                match v.cmp(&u) {
                    Equal => None,
                    Less => Some(u - 1),
                    Greater => Some(u),
                }
            })
            .collect::<Vec<_>>();
    }

    #[allow(dead_code)]
    pub fn replace(&mut self, old: VertexId, new: VertexId) {
        self.0 = self
            .0
            .clone()
            .into_iter()
            .filter_map(|v| {
                if v == new {
                    None
                } else if v == old {
                    Some(new)
                } else {
                    Some(v)
                }
            })
            .collect();
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

impl From<Vec<[VertexId; 2]>> for Cycle {
    fn from(mut edges: Vec<[VertexId; 2]>) -> Self {
        let mut first = false;
        let mut face = vec![edges[0][0]];
        while !edges.is_empty() {
            let v = if first {
                *face.first().unwrap()
            } else {
                *face.last().unwrap()
            };
            if let Some(i) = edges.iter().position(|e| e.contains(&v)) {
                let next = if edges[i][0] == v {
                    edges[i][1]
                } else {
                    edges[i][0]
                };
                if !face.contains(&next) {
                    face.push(next);
                }
                edges.remove(i);
            } else {
                first ^= true;
            }
        }
        Self(face)
    }
}
