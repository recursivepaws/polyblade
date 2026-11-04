mod cycle;
#[cfg(test)]
mod polydex_gen;
use crate::polyhedron::{FaceId, VertexId};
use crate::render::pipeline::ShapeVertex;
pub use cycle::*;
use std::{
    collections::{HashMap, HashSet},
    ops::{Index, IndexMut},
};
use ultraviolet::{Vec3, Vec4};

use super::Distance;
use super::topology::undirected;

#[derive(Default, Debug, Clone)]
pub(in super::super) struct Cycles {
    // Circular lists of Vertex Ids representing faces
    cycles: Vec<Cycle>,
    /// Stable identity per face, parallel to `cycles`; survives sorts and operations.
    ids: Vec<FaceId>,
}

impl Cycles {
    pub fn new(cycles: Vec<Vec<VertexId>>, ids: Vec<FaceId>) -> Self {
        debug_assert_eq!(cycles.len(), ids.len());
        Self {
            cycles: cycles.into_iter().map(Cycle).collect(),
            ids,
        }
    }

    /// Stable face ids, parallel to the cycle list.
    pub fn ids(&self) -> &[FaceId] {
        &self.ids
    }

    /// Canonical face order: more sides first, then a more uniform neighborhood, then sorted vertices.
    /// Total on these polyhedra (distinct faces have distinct vertex sets), so face 0 is deterministic.
    pub fn sort(&mut self) {
        let raw: Vec<Vec<VertexId>> = self.cycles.iter().map(|c| c.0.clone()).collect();
        let neighbor_uniformity: Vec<usize> = neighbor_type_signatures(&raw)
            .iter()
            .map(|sig| sig.iter().collect::<HashSet<_>>().len())
            .collect();
        let mut scored: Vec<(Cycle, FaceId, usize)> = std::mem::take(&mut self.cycles)
            .into_iter()
            .zip(std::mem::take(&mut self.ids))
            .zip(neighbor_uniformity)
            .map(|((c, id), u)| (c, id, u))
            .collect();
        scored.sort_by_key(|(c, _, uniformity)| {
            let mut sorted_vertices = c.0.clone();
            sorted_vertices.sort();
            (usize::MAX - c.len(), *uniformity, sorted_vertices)
        });
        for (c, id, _) in scored {
            self.cycles.push(c);
            self.ids.push(id);
        }
        #[cfg(debug_assertions)]
        self.assert_oriented();
    }

    /// Debug invariant: every undirected edge is traversed exactly once in each direction.
    #[cfg(debug_assertions)]
    fn assert_oriented(&self) {
        let mut directed: HashSet<(VertexId, VertexId)> = HashSet::new();
        for cycle in self.iter() {
            for k in 0..cycle.len() {
                assert!(
                    directed.insert((cycle[k], cycle[k + 1])),
                    "edge traversed twice in the same direction"
                );
            }
        }
        for &(a, b) in &directed {
            assert!(
                directed.contains(&(b, a)),
                "edge missing its opposite traversal"
            );
        }
    }

    /// Rediscovers faces from the distance matrix, minting fresh ids.
    /// Only seed construction uses this, operations build their cycles explicitly.
    /// Orientation is made consistent and canonically handed here, see `orient_faces`.
    pub(super) fn discover(distance: &Distance, next_face_id: &mut FaceId) -> Self {
        let mut raw = chordless_cycles(distance);
        orient_faces(&mut raw);
        let ids = raw
            .iter()
            .map(|_| {
                let id = *next_face_id;
                *next_face_id += 1;
                id
            })
            .collect();
        let mut cycles = Cycles::new(raw, ids);
        cycles.sort();
        cycles
    }

    pub fn len(&self) -> usize {
        self.cycles.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Cycle> {
        self.cycles.iter()
    }
    /// Neighbors of `v` in cyclic order, wound opposite to the incident faces.
    /// Faces built directly from this ring stay consistently oriented.
    pub fn sorted_connections(&self, v: VertexId) -> Vec<VertexId> {
        // A face traversing (a, v, b) forces the ring step b -> a.
        let steps = self
            .iter()
            .filter_map(|cycle| {
                cycle
                    .iter()
                    .position(|&x| x == v)
                    .map(|p| (cycle[p + 1], cycle[p + cycle.len() - 1]))
            })
            .collect::<Vec<(VertexId, VertexId)>>();
        let successor: HashMap<VertexId, VertexId> = steps.iter().copied().collect();

        let mut ring = Vec::with_capacity(steps.len());
        let mut next = steps[0].0;
        for _ in 0..steps.len() {
            ring.push(next);
            next = successor[&next];
        }
        ring
    }
    pub fn shape_vertices(&self) -> Vec<ShapeVertex> {
        let barycentric = [Vec3::unit_x(), Vec3::unit_y(), Vec3::unit_z()];
        self.iter()
            .map(|cycle| {
                let sides: Vec4 = match cycle.len() {
                    3 => Vec3::new(1.0, 1.0, 1.0),
                    4 => Vec3::new(1.0, 0.0, 1.0),
                    _ => Vec3::new(0.0, 1.0, 0.0),
                }
                .into();

                let b_shapes: Vec<ShapeVertex> = barycentric
                    .iter()
                    .map(|&b| ShapeVertex {
                        barycentric: b.into(),
                        sides,
                    })
                    .collect();

                match cycle.len() {
                    3 => b_shapes.clone(),
                    4 => (0..6)
                        .map(|i| ShapeVertex {
                            barycentric: barycentric[i % 3].into(),
                            sides,
                        })
                        .collect(),
                    _ => vec![b_shapes; cycle.len()].concat(),
                }
            })
            .collect::<Vec<Vec<ShapeVertex>>>()
            .concat()
    }

    /// For each face, the sorted multiset of its edge-adjacent neighbors' side-counts.
    pub fn neighbor_signatures(&self) -> Vec<Vec<usize>> {
        let cycles: Vec<Vec<VertexId>> = self.cycles.iter().map(|c| c.0.clone()).collect();
        neighbor_type_signatures(&cycles)
    }
}

impl Index<usize> for Cycles {
    type Output = Cycle;

    fn index(&self, index: usize) -> &Self::Output {
        &self.cycles[index.rem_euclid(self.cycles.len())]
    }
}

impl IndexMut<usize> for Cycles {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let len = self.cycles.len();
        &mut self.cycles[index.rem_euclid(len)]
    }
}

impl Cycles {
    /// Replays the same merge sequence as `Distance::contract_edges` on the face list.
    /// Survivors keep their ids and faces that degenerate below 3 vertices are dropped.
    pub fn contract_edges(&mut self, edges: Vec<[VertexId; 2]>) {
        crate::polyhedron::contract_edge_indices(edges, |v, u| {
            // Merge `v` into `u` in every face.
            // Rebuild in a single pass; keep each surviving cycle paired with its id.
            let cycles = std::mem::take(&mut self.cycles);
            let ids = std::mem::take(&mut self.ids);
            for (mut cycle, id) in cycles.into_iter().zip(ids) {
                if cycle.contract_vertex(v, u) {
                    self.cycles.push(cycle);
                    self.ids.push(id);
                }
            }
        });
    }
}

/// For each face, the sorted multiset of adjacent side-counts; shared by the sort key below and `Cycles::neighbor_signatures`.
fn neighbor_type_signatures(cycles: &[Vec<VertexId>]) -> Vec<Vec<usize>> {
    let directed = super::topology::directed_edge_faces(cycles);
    cycles
        .iter()
        .map(|cycle| {
            let n = cycle.len();
            let mut sides: Vec<usize> = (0..n)
                .map(|k| {
                    let (a, b) = (cycle[k], cycle[(k + 1) % n]);
                    cycles[directed[&(b, a)]].len()
                })
                .collect();
            sides.sort_unstable();
            sides
        })
        .collect()
}

/// Flips faces so every shared edge is traversed once in each direction.
/// Assumes a connected closed 2-manifold; which global handedness wins is arbitrary.
fn orient_faces(cycles: &mut [Vec<VertexId>]) {
    let mut edge_faces: HashMap<[VertexId; 2], Vec<usize>> = HashMap::new();
    for (f, cycle) in cycles.iter().enumerate() {
        for k in 0..cycle.len() {
            let edge = undirected(cycle[k], cycle[(k + 1) % cycle.len()]);
            edge_faces.entry(edge).or_default().push(f);
        }
    }
    debug_assert!(
        edge_faces.values().all(|fs| fs.len() == 2),
        "edge not bordering exactly two faces"
    );

    let traverses = |cycle: &[VertexId], a: VertexId, b: VertexId| {
        (0..cycle.len()).any(|k| cycle[k] == a && cycle[(k + 1) % cycle.len()] == b)
    };
    let mut visited = vec![false; cycles.len()];
    let mut queue = std::collections::VecDeque::from([0]);
    visited[0] = true;
    while let Some(f) = queue.pop_front() {
        let cycle = cycles[f].clone();
        for k in 0..cycle.len() {
            let (a, b) = (cycle[k], cycle[(k + 1) % cycle.len()]);
            let g = *edge_faces[&undirected(a, b)]
                .iter()
                .find(|&&g| g != f)
                .expect("edge borders its face twice");
            if !visited[g] {
                if traverses(&cycles[g], a, b) {
                    cycles[g].reverse();
                }
                visited[g] = true;
                queue.push_back(g);
            } else {
                debug_assert!(!traverses(&cycles[g], a, b), "nonorientable face pairing");
            }
        }
    }
    debug_assert!(visited.iter().all(|&v| v), "face adjacency not connected");

    // Canonicalize the one global handedness bit so discovery stays deterministic.
    // The lexicographically smallest face must step from its least vertex to its lesser neighbor.
    let f = (0..cycles.len())
        .min_by_key(|&f| {
            let mut vs = cycles[f].clone();
            vs.sort_unstable();
            vs
        })
        .unwrap();
    let n = cycles[f].len();
    let m = (0..n).min_by_key(|&k| cycles[f][k]).unwrap();
    if cycles[f][(m + 1) % n] > cycles[f][(m + n - 1) % n] {
        for cycle in cycles.iter_mut() {
            cycle.reverse();
        }
    }
}

/// Chordless-cycle face search over the distance matrix; expensive, unordered output.
fn chordless_cycles(distance: &Distance) -> Vec<Vec<VertexId>> {
    let mut triplets: Vec<Vec<_>> = Default::default();
    let mut cycles: HashSet<Vec<_>> = Default::default();
    // find all the triplets
    for u in 0..distance.order() {
        for x in (u + 1)..distance.order() {
            for y in (x + 1)..distance.order() {
                if distance[[u, x]] == 1 && distance[[u, y]] == 1 {
                    if distance[[x, y]] == 1 {
                        cycles.insert(vec![x, u, y]);
                    } else {
                        triplets.push(vec![x, u, y]);
                    }
                }
            }
        }
    }

    // while there are unparsed triplets
    while !triplets.is_empty() && (cycles.len() as i64) < distance.face_count() {
        let p = triplets.remove(0);

        // for each v adjacent to u_t
        for v in distance.neighbors(p[p.len() - 1]) {
            if v > p[1] {
                let adj_v = distance.neighbors(v);
                // if v is not a neighbor of u_2..u_t-1
                if !p[1..p.len() - 1].iter().any(|i| adj_v.contains(i)) {
                    let new = [p.clone(), vec![v]].concat();
                    if distance.neighbors(p[0]).contains(&v) {
                        if distance.cycle_is_face(new.clone()) {
                            cycles.insert(new);
                        }
                    } else {
                        triplets.push(new);
                    }
                }
            }
        }
    }

    cycles.into_iter().collect()
}
