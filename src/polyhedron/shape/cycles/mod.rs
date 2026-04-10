mod cycle;
use crate::{polyhedron::VertexId, render::pipeline::ShapeVertex};
pub use cycle::*;
use std::{
    collections::HashSet,
    ops::{Index, IndexMut},
};
use ultraviolet::{Vec3, Vec4};

use super::Distance;

#[derive(Default, Debug, Clone)]
pub(in super::super) struct Cycles {
    // Circular lists of Vertex Ids representing faces
    cycles: Vec<Cycle>,
}

impl Cycles {
    pub fn new(cycles: Vec<Vec<VertexId>>) -> Self {
        Self {
            cycles: cycles.into_iter().map(Cycle).collect(),
        }
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.cycles.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Cycle> {
        self.cycles.iter()
    }

    #[allow(dead_code)]
    pub fn into_iter(self) -> std::vec::IntoIter<Cycle> {
        self.cycles.into_iter()
    }
    /// Returns the
    pub fn sorted_connections(&self, v: VertexId) -> Vec<VertexId> {
        // We only care about cycles that contain the vertex
        let mut relevant = self
            .iter()
            .filter_map(move |cycle| {
                cycle
                    .iter()
                    .position(|&x| x == v)
                    .map(|p| [cycle[p + cycle.len() - 1], cycle[p + 1]])
            })
            .collect::<Vec<[VertexId; 2]>>();

        let mut sorted_connections = vec![relevant[0][0]];
        loop {
            let previous = sorted_connections.last().unwrap();
            match relevant
                .iter()
                .position(|[v, u]| v == previous || u == previous)
            {
                Some(i) => {
                    let [v, u] = relevant.remove(i);
                    let next = if v == *previous { u } else { v };
                    sorted_connections.push(next);
                }
                None => {
                    break;
                }
            }
        }

        sorted_connections[1..].to_vec()
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
    #[allow(dead_code)]
    pub fn delete(&mut self, v: VertexId) {
        for cycle in &mut self.cycles {
            cycle.delete(v);
        }
    }

    /// Replace all occurrence of one vertex with another
    #[allow(dead_code)]
    pub fn replace(&mut self, old: VertexId, new: VertexId) {
        for cycle in &mut self.cycles {
            cycle.replace(old, new);
        }
    }
}

impl From<&Distance> for Cycles {
    fn from(distance: &Distance) -> Self {
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

        let mut cycles = cycles.into_iter().collect::<Vec<_>>();
        cycles.sort_by_key(|c| usize::MAX - c.len());
        Cycles::new(cycles)
    }
}
