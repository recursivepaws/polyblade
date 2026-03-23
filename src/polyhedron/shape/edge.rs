use std::collections::{HashMap, HashSet};
use std::ops::Index;

use crate::polyhedron::VertexId;

// --- EdgeKey ---

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct EdgeKey([VertexId; 2]);

impl EdgeKey {
    pub fn new(a: VertexId, b: VertexId) -> Self {
        if a <= b {
            Self([a, b])
        } else {
            Self([b, a])
        }
    }

    pub fn inner(&self) -> [VertexId; 2] {
        self.0
    }
}

impl From<[VertexId; 2]> for EdgeKey {
    fn from([a, b]: [VertexId; 2]) -> Self {
        EdgeKey::new(a, b)
    }
}

// impl From<(VertexId, VertexId)> for EdgeKey {
//     fn from((a, b): (VertexId, VertexId)) -> Self {
//         EdgeKey::new(a, b)
//     }
// }

// --- EdgeMap ---

#[derive(Default)]
pub struct EdgeMap<T>(HashMap<EdgeKey, T>);

impl<T> EdgeMap<T> {
    pub fn get(&self, key: impl Into<EdgeKey>) -> Option<&T> {
        self.0.get(&key.into())
    }

    pub fn get_mut(&mut self, key: impl Into<EdgeKey>) -> Option<&mut T> {
        self.0.get_mut(&key.into())
    }

    pub fn insert(&mut self, key: impl Into<EdgeKey>, value: T) -> Option<T> {
        self.0.insert(key.into(), value)
    }

    pub fn remove(&mut self, key: impl Into<EdgeKey>) -> Option<T> {
        self.0.remove(&key.into())
    }

    pub fn contains_key(&self, key: impl Into<EdgeKey>) -> bool {
        self.0.contains_key(&key.into())
    }

    pub fn keys(&self) -> impl Iterator<Item = &[VertexId; 2]> {
        self.0.keys().map(|x| &x.0)
    }
}

impl<T, K: Into<EdgeKey>> Index<K> for EdgeMap<T> {
    type Output = T;

    fn index(&self, key: K) -> &T {
        &self.0[&key.into()]
    }
}

// --- EdgeSet ---

#[derive(Default)]
pub struct EdgeSet(HashSet<EdgeKey>);

impl EdgeSet {
    pub fn contains(&self, key: impl Into<EdgeKey>) -> bool {
        self.0.contains(&key.into())
    }

    pub fn insert(&mut self, key: impl Into<EdgeKey>) -> bool {
        self.0.insert(key.into())
    }

    pub fn remove(&mut self, key: impl Into<EdgeKey>) -> bool {
        self.0.remove(&key.into())
    }

    pub fn iter(&self) -> impl Iterator<Item = [VertexId; 2]> + '_ {
        self.0.iter().map(|x| &x.0).copied()
    }
}
