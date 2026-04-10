use crate::bones::{Edge, VertexId};
use rustc_hash::FxHashSet as HashSet;
use std::{
    hash::Hash,
    ops::{Index, IndexMut},
    slice::SliceIndex,
    vec::IntoIter,
};

#[derive(Debug, Default, Clone, PartialOrd, Ord)]
pub struct Face(Vec<VertexId>);

impl Face {
    pub fn new(vertices: Vec<VertexId>) -> Self {
        Self(vertices)
    }
    pub fn contains(&self, other: &Face) -> bool {
        other.0.iter().all(|v| self.0.contains(v))
    }

    pub fn containz(&self, value: &VertexId) -> bool {
        self.0.contains(value)
    }

    pub fn edges(&self) -> HashSet<Edge> {
        let mut edges = HashSet::default();
        for i in 0..self.0.len() {
            edges.insert((self.0[i], self.0[(i + 1) % self.0.len()]).into());
        }
        edges
    }

    pub fn replace(&mut self, old: VertexId, new: VertexId) {
        if self.0.contains(&new) && self.0.contains(&old) {
            self.remove(self.0.iter().position(|&x| x == old).unwrap());
        } else {
            self.0 = self
                .0
                .clone()
                .into_iter()
                .map(|x| if x == old { new } else { x })
                .collect();
        }
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, usize> {
        self.0.iter()
    }

    pub fn remove(&mut self, index: usize) -> VertexId {
        self.0.remove(index)
    }

    pub fn insert(&mut self, index: usize, v: VertexId) {
        self.0.insert(index, v)
    }

    pub fn push(&mut self, value: VertexId) {
        self.0.push(value)
    }
}

impl From<HashSet<Edge>> for Face {
    fn from(value: HashSet<Edge>) -> Self {
        let mut edges: Vec<Edge> = value.into_iter().collect();
        let mut first = false;
        let mut face = vec![edges[0].v()];
        while !edges.is_empty() {
            let v = if first {
                *face.first().unwrap()
            } else {
                *face.last().unwrap()
            };
            if let Some(i) = edges.iter().position(|e| e.contains(v)) {
                let next = edges[i].other(v).unwrap();
                if !face.contains(&next) {
                    face.push(next);
                }
                edges.remove(i);
            } else {
                first ^= true;
            }
        }
        Self::new(face)
    }
}

impl<Idx> Index<Idx> for Face
where
    Idx: SliceIndex<[usize]>,
{
    type Output = Idx::Output;

    #[inline(always)]
    fn index(&self, index: Idx) -> &Self::Output {
        self.0.index(index)
    }
}
impl<Idx> IndexMut<Idx> for Face
where
    Idx: SliceIndex<[usize], Output = usize>,
{
    #[inline]
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl IntoIterator for Face {
    type Item = VertexId;
    type IntoIter = IntoIter<VertexId>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<VertexId> for Face {
    fn from_iter<I: IntoIterator<Item = VertexId>>(iter: I) -> Self {
        let mut c = Face::new(vec![]);
        for i in iter {
            c.0.push(i);
        }
        c
    }
}

impl PartialEq for Face {
    fn eq(&self, other: &Self) -> bool {
        self.contains(other) && self.0.len() == other.0.len()
    }
}

impl Eq for Face {}
impl Hash for Face {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut edges = self.edges().into_iter().collect::<Vec<_>>();
        edges.sort();
        for edge in edges {
            edge.hash(state);
        }
    }
}
