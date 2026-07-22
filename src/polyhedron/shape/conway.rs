use super::{Cycles, Distance, Shape};
use crate::polyhedron::{FaceId, VertexId};
use std::collections::{HashMap, HashSet};

impl Shape {
    pub fn split_vertex(&mut self, v: VertexId) -> Vec<[usize; 2]> {
        let sc = self.cycles.sorted_connections(v);
        let edges = self.distance.split_vertex(v, sc.clone());
        // The ring edges are [corner_i, corner_i+1], where corner_i stays adjacent to sc[i].
        let corners: Vec<VertexId> = edges.iter().map(|&[a, _]| a).collect();

        // Faces containing v keep their id, with v replaced by prev's then next's corner.
        let mut new_cycles: Vec<Vec<VertexId>> = Vec::with_capacity(self.cycles.len() + 1);
        let mut new_ids: Vec<FaceId> = Vec::with_capacity(self.cycles.len() + 1);
        for (i, cycle) in self.cycles.iter().enumerate() {
            let mut face: Vec<VertexId> = cycle.iter().copied().collect();
            if let Some(k) = face.iter().position(|&x| x == v) {
                let n = face.len();
                let prev = face[(k + n - 1) % n];
                let next = face[(k + 1) % n];
                let j = sc.iter().position(|&x| x == prev).unwrap();
                let m = sc.iter().position(|&x| x == next).unwrap();
                face.splice(k..=k, [corners[j], corners[m]]);
            }
            new_cycles.push(face);
            new_ids.push(self.cycles.ids()[i]);
        }
        // The corner ring itself is the new vertex-figure face.
        new_cycles.push(corners);
        new_ids.push(self.next_face_id);
        self.next_face_id += 1;

        self.cycles = Cycles::new(new_cycles, new_ids);
        self.cycles.sort();
        self.assert_cycles_match_discovery();
        edges
    }

    /// `t` full truncation: one new vertex per (vertex, incident-edge) corner.
    /// Returns the vertex-figure edges (so `ambo` contracts the rest) and each new vertex's origin for render re-seeding.
    pub fn truncate(&mut self) -> (Vec<[VertexId; 2]>, Vec<VertexId>) {
        // Index every (vertex, neighbor) corner; `corner[(v, u)]` is the new vertex there.
        let mut corner: HashMap<(VertexId, VertexId), VertexId> = HashMap::new();
        let mut parents: Vec<VertexId> = Vec::new();
        let mut vertex_order: Vec<Vec<VertexId>> = Vec::with_capacity(self.order());
        for v in self.vertices() {
            let sc = self.cycles.sorted_connections(v);
            for &u in &sc {
                corner.insert((v, u), parents.len());
                parents.push(v);
            }
            vertex_order.push(sc);
        }

        let mut distance = Distance::new(parents.len());
        // Vertex-figure d-gon: link a vertex's corners in cyclic neighbor order.
        // The truncated faces (2n-gons) emerge for free from these plus the original edges.
        let mut new_edges = Vec::new();
        for (v, sc) in vertex_order.iter().enumerate() {
            let d = sc.len();
            for i in 0..d {
                let edge = [corner[&(v, sc[i])], corner[&(v, sc[(i + 1) % d])]];
                distance.connect(edge);
                new_edges.push(edge);
            }
        }
        // Original edges: each keeps its two endpoint corners joined.
        for [v, u] in self.edges() {
            distance.connect([corner[&(v, u)], corner[&(u, v)]]);
        }

        // Each original face persists as the 2n-gon over its corner copies, keeping its id.
        // Consecutive corners alternate between original-edge crossings and vertex-figure edges.
        let mut new_cycles: Vec<Vec<VertexId>> = Vec::new();
        let mut new_ids: Vec<FaceId> = Vec::new();
        for (i, cycle) in self.cycles.iter().enumerate() {
            let gon = (0..cycle.len())
                .flat_map(|k| {
                    let (a, b) = (cycle[k], cycle[k + 1]);
                    [corner[&(a, b)], corner[&(b, a)]]
                })
                .collect();
            new_cycles.push(gon);
            new_ids.push(self.cycles.ids()[i]);
        }
        // Each original vertex spawns its vertex-figure d-gon: a genuinely new face.
        for (v, sc) in vertex_order.iter().enumerate() {
            new_cycles.push(sc.iter().map(|&u| corner[&(v, u)]).collect());
            new_ids.push(self.next_face_id);
            self.next_face_id += 1;
        }

        self.distance = distance;
        self.cycles = Cycles::new(new_cycles, new_ids);
        self.cycles.sort();
        self.recompute_metrics();
        self.assert_cycles_match_discovery();
        (new_edges, parents)
    }

    pub fn contract_edges(&mut self, edges: Vec<[VertexId; 2]>) {
        self.distance.contract_edges(edges.clone());
        // Both walks replay the same merge sequence, so the face list tracks the matrix exactly.
        self.cycles.contract_edges(edges);
        self.cycles.sort();
        self.recompute_metrics();
        self.assert_cycles_match_discovery();
    }

    pub fn kis(&mut self, degree: Option<usize>) -> Vec<[VertexId; 2]> {
        let edges = self.distance.edges().collect();

        let mut new_cycles: Vec<Vec<VertexId>> = Vec::new();
        let mut new_ids: Vec<FaceId> = Vec::new();
        for i in 0..self.cycles.len() {
            let face: Vec<VertexId> = self.cycles[i].iter().copied().collect();
            let id = self.cycles.ids()[i];
            if degree.is_some_and(|d| face.len() != d) {
                // Untouched face persists as-is.
                new_cycles.push(face);
                new_ids.push(id);
                continue;
            }
            // Raise an apex over the face; it splits into n triangles, each carved from `id`.
            let v = self.distance.insert();
            let n = face.len();
            for k in 0..n {
                self.distance.connect([v, face[k]]);
                new_cycles.push(vec![v, face[k], face[(k + 1) % n]]);
                new_ids.push(self.next_face_id);
                self.birth_parents.insert(self.next_face_id, id);
                self.next_face_id += 1;
            }
        }

        self.cycles = Cycles::new(new_cycles, new_ids);
        self.cycles.sort();
        self.recompute_metrics();
        // No discovery oracle here because discovery falsely admits the covered original triangles.
        // The explicit construction is the ground truth, verified by count assertions in tests.
        edges
    }

    /// `e` expand / cantellation: one new vertex per original vertex-face corner.
    /// Returns each new vertex's origin (for render re-seeding) and the face-figure edges to contract for the dual.
    pub fn expand(&mut self) -> (Vec<VertexId>, Vec<[VertexId; 2]>) {
        let cycles: Vec<Vec<VertexId>> = self
            .cycles
            .iter()
            .map(|c| c.iter().copied().collect())
            .collect();
        let old_ids: Vec<FaceId> = self.cycles.ids().to_vec();

        // Index every (face, corner) incidence; `c[f][i]` is the new vertex there.
        let mut c: Vec<Vec<VertexId>> = Vec::with_capacity(cycles.len());
        let mut parents: Vec<VertexId> = Vec::new();
        for cycle in &cycles {
            let row = cycle
                .iter()
                .map(|&v| {
                    parents.push(v);
                    parents.len() - 1
                })
                .collect();
            c.push(row);
        }

        // Position of a vertex within a face's cycle.
        let pos = |f: usize, v: VertexId| cycles[f].iter().position(|&x| x == v).unwrap();
        // Which two faces each original edge borders.
        let mut edge_faces: HashMap<[VertexId; 2], Vec<usize>> = HashMap::new();
        for (f, cycle) in cycles.iter().enumerate() {
            let n = cycle.len();
            for k in 0..n {
                let (a, b) = (cycle[k], cycle[(k + 1) % n]);
                let edge = if a < b { [a, b] } else { [b, a] };
                edge_faces.entry(edge).or_default().push(f);
            }
        }

        let mut distance = Distance::new(parents.len());
        // Face-figure edges: the original n-gon, using this face's corner copies.
        // Contracting these collapses each face to a point, giving the dual.
        let mut face_edges = Vec::new();
        for (f, cycle) in cycles.iter().enumerate() {
            let n = cycle.len();
            for k in 0..n {
                let edge = [c[f][k], c[f][(k + 1) % n]];
                distance.connect(edge);
                face_edges.push(edge);
            }
        }
        // Vertex-figure rungs: link the two faces' copies of each endpoint.
        for (edge, faces) in &edge_faces {
            if faces.len() != 2 {
                continue;
            }
            let [f, g] = [faces[0], faces[1]];
            for &v in edge {
                distance.connect([c[f][pos(f, v)], c[g][pos(g, v)]]);
            }
        }

        // Each original face persists as its corner-copy n-gon, keeping its id.
        let mut new_cycles: Vec<Vec<VertexId>> = c.clone();
        let mut new_ids: Vec<FaceId> = old_ids;
        // Each original edge spawns a quad, interleaved so each face's copy pair stays adjacent.
        let mut seen: HashSet<[VertexId; 2]> = HashSet::new();
        for (f, cycle) in cycles.iter().enumerate() {
            let n = cycle.len();
            for k in 0..n {
                let (a, b) = (cycle[k], cycle[(k + 1) % n]);
                let edge = if a < b { [a, b] } else { [b, a] };
                if !seen.insert(edge) {
                    continue;
                }
                let faces = &edge_faces[&edge];
                if faces.len() != 2 {
                    continue;
                }
                let g = if faces[0] == f { faces[1] } else { faces[0] };
                new_cycles.push(vec![
                    c[f][pos(f, a)],
                    c[f][pos(f, b)],
                    c[g][pos(g, b)],
                    c[g][pos(g, a)],
                ]);
                new_ids.push(self.next_face_id);
                self.next_face_id += 1;
            }
        }
        // Each original vertex spawns its vertex-figure by walking the faces around v.
        for v in 0..self.order() {
            let f0 = (0..cycles.len())
                .find(|&f| cycles[f].contains(&v))
                .expect("vertex belongs to no face");
            let mut figure = Vec::new();
            let mut f = f0;
            // Enter f0 via its edge (prev, v); the walk exits via (v, next) each step.
            let mut entry = {
                let k = pos(f0, v);
                let prev = cycles[f0][(k + cycles[f0].len() - 1) % cycles[f0].len()];
                if prev < v { [prev, v] } else { [v, prev] }
            };
            loop {
                figure.push(c[f][pos(f, v)]);
                let k = pos(f, v);
                let next = cycles[f][(k + 1) % cycles[f].len()];
                let prev = cycles[f][(k + cycles[f].len() - 1) % cycles[f].len()];
                // Exit via whichever of v's two edges in f we didn't enter through.
                let e_next = if next < v { [next, v] } else { [v, next] };
                let e_prev = if prev < v { [prev, v] } else { [v, prev] };
                let exit = if e_next == entry { e_prev } else { e_next };
                let faces = &edge_faces[&exit];
                debug_assert_eq!(faces.len(), 2, "open edge at vertex figure");
                f = if faces[0] == f { faces[1] } else { faces[0] };
                entry = exit;
                if f == f0 {
                    break;
                }
            }
            new_cycles.push(figure);
            new_ids.push(self.next_face_id);
            self.next_face_id += 1;
        }

        self.distance = distance;
        self.cycles = Cycles::new(new_cycles, new_ids);
        self.cycles.sort();
        self.recompute_metrics();
        self.assert_cycles_match_discovery();
        (parents, face_edges)
    }

    pub fn chamfer(&mut self) {
        let originals = self.edges().collect::<Vec<_>>();
        let cycles: Vec<Vec<VertexId>> = self
            .cycles
            .iter()
            .map(|c| c.iter().copied().collect())
            .collect();
        let old_ids: Vec<FaceId> = self.cycles.ids().to_vec();

        // Shrink each face: one new vertex per corner, tethered to the original and ringed together.
        let mut c: Vec<Vec<VertexId>> = Vec::with_capacity(cycles.len());
        for cycle in &cycles {
            let row: Vec<VertexId> = cycle
                .iter()
                .map(|&v| {
                    let u = self.distance.insert();
                    self.distance.connect([v, u]);
                    u
                })
                .collect();
            for k in 0..row.len() {
                self.distance.connect([row[k], row[(k + 1) % row.len()]]);
            }
            c.push(row);
        }
        for edge in originals {
            self.distance.disconnect(edge);
        }

        let pos = |f: usize, v: VertexId| cycles[f].iter().position(|&x| x == v).unwrap();
        let mut edge_faces: HashMap<[VertexId; 2], Vec<usize>> = HashMap::new();
        for (f, cycle) in cycles.iter().enumerate() {
            let n = cycle.len();
            for k in 0..n {
                let (a, b) = (cycle[k], cycle[(k + 1) % n]);
                let edge = if a < b { [a, b] } else { [b, a] };
                edge_faces.entry(edge).or_default().push(f);
            }
        }

        // Each original face persists as its shrunk copy, keeping its id.
        let mut new_cycles: Vec<Vec<VertexId>> = c.clone();
        let mut new_ids: Vec<FaceId> = old_ids;
        // Each original edge spawns a hexagon through both faces' shrunk copies.
        let mut seen: HashSet<[VertexId; 2]> = HashSet::new();
        for (f, cycle) in cycles.iter().enumerate() {
            let n = cycle.len();
            for k in 0..n {
                let (a, b) = (cycle[k], cycle[(k + 1) % n]);
                let edge = if a < b { [a, b] } else { [b, a] };
                if !seen.insert(edge) {
                    continue;
                }
                let faces = &edge_faces[&edge];
                if faces.len() != 2 {
                    continue;
                }
                let g = if faces[0] == f { faces[1] } else { faces[0] };
                new_cycles.push(vec![
                    a,
                    c[f][pos(f, a)],
                    c[f][pos(f, b)],
                    b,
                    c[g][pos(g, b)],
                    c[g][pos(g, a)],
                ]);
                new_ids.push(self.next_face_id);
                self.next_face_id += 1;
            }
        }

        self.cycles = Cycles::new(new_cycles, new_ids);
        self.cycles.sort();
        self.recompute_metrics();
        self.assert_cycles_match_discovery();
    }
}
