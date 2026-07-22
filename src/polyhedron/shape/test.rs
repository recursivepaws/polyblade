use super::*;

impl Shape {
    pub fn floyd(&mut self) {
        self.distance.floyd();
    }
}

#[test]
fn truncate_keeps_face_ids_on_2n_gons() {
    let mut cube = Shape::prism(4);
    let old: Vec<(FaceId, usize)> = cube
        .cycles
        .ids()
        .iter()
        .zip(cube.cycles.iter())
        .map(|(&id, c)| (id, c.len()))
        .collect();

    cube.truncate();

    // Every original face survives under its id, with doubled side count.
    for (id, n) in old {
        let i = cube
            .cycles
            .ids()
            .iter()
            .position(|&x| x == id)
            .unwrap_or_else(|| panic!("face id {id} lost by truncation"));
        assert_eq!(cube.cycles[i].len(), 2 * n, "2n-gon side count for id {id}");
    }
}

#[test]
fn expand_keeps_face_ids_on_corner_copies() {
    let mut cube = Shape::prism(4);
    let old: Vec<(FaceId, usize)> = cube
        .cycles
        .ids()
        .iter()
        .zip(cube.cycles.iter())
        .map(|(&id, c)| (id, c.len()))
        .collect();

    cube.expand();

    // Every original face survives under its id, same side count (its corner-copy n-gon).
    for (id, n) in old {
        let i = cube
            .cycles
            .ids()
            .iter()
            .position(|&x| x == id)
            .unwrap_or_else(|| panic!("face id {id} lost by expansion"));
        assert_eq!(
            cube.cycles[i].len(),
            n,
            "corner-copy side count for id {id}"
        );
    }
}

#[test]
fn op_chains_match_discovery() {
    // The debug oracle inside each op asserts cycles match discovery on every chain here.
    for seed in [
        Shape::pyramid(3),
        Shape::prism(4),
        Shape::anti_prism(3),
        Shape::anti_prism(5),
    ] {
        let mut s = seed.clone();
        s.truncate();
        s.expand();

        // Dual chain: expand, then contract the face-figure edges.
        let mut s = seed.clone();
        let (_, face_edges) = s.expand();
        s.contract_edges(face_edges);

        // Double dual returns to the seed's face counts.
        let (_, face_edges) = s.expand();
        s.contract_edges(face_edges);
        assert_eq!(s.cycles.len(), seed.cycles.len(), "dd face count");
    }
}

#[test]
fn chamfer_cube_counts_and_ids() {
    let mut cube = Shape::prism(4);
    let old_ids: Vec<FaceId> = cube.cycles.ids().to_vec();

    cube.chamfer();

    // Chamfered cube: V = 8 + 2E = 32, E = 4E = 48, F = 6 + 12 = 18.
    assert_eq!(cube.order(), 32, "vertex count");
    assert_eq!(cube.edges().count(), 48, "edge count");
    assert_eq!(cube.cycles.len(), 18, "face count");
    // Original faces persist (shrunk) under their ids, still squares.
    for id in old_ids {
        let i = cube.cycles.ids().iter().position(|&x| x == id).unwrap();
        assert_eq!(cube.cycles[i].len(), 4, "shrunk face keeps side count");
    }
    let hexes = cube.cycles.iter().filter(|c| c.len() == 6).count();
    assert_eq!(hexes, 12, "one hexagon per original edge");
}

#[test]
fn kis_children_record_their_parent() {
    let mut tetra = Shape::pyramid(3);
    let old_ids: Vec<FaceId> = tetra.cycles.ids().to_vec();

    tetra.kis(None);

    // Kis tetrahedron: every face is a fresh triangle carved from an original face.
    assert_eq!(tetra.cycles.len(), 12, "face count");
    for &id in tetra.cycles.ids() {
        let parent = tetra.birth_parents.get(&id);
        assert!(
            parent.is_some_and(|p| old_ids.contains(p)),
            "face {id} must record an original parent"
        );
    }
}

#[test]
fn contract_face_ring_keeps_survivor_ids() {
    // Contracting a whole cube face ring chains merges until its last edge degenerates to [u, u].
    // The face collapses to a point, leaving a square pyramid.
    let mut cube = Shape::prism(4);
    let ring: Vec<[VertexId; 2]> = {
        let cycle = &cube.cycles[0];
        (0..cycle.len()).map(|i| [cycle[i], cycle[i + 1]]).collect()
    };
    let survivor_ids: Vec<FaceId> = cube
        .cycles
        .ids()
        .iter()
        .copied()
        .skip(1) // face 0 is the one being collapsed
        .collect();

    cube.contract_edges(ring);

    // Square pyramid: V=5, E=8, F=5 (the opposite square + 4 side squares pinched to triangles).
    assert_eq!(cube.order(), 5, "vertex count");
    assert_eq!(cube.edges().count(), 8, "edge count");
    assert_eq!(cube.cycles.len(), 5, "face count");
    // The collapsed face's id died; every other face survived with its id intact.
    let mut expected = survivor_ids;
    expected.sort_unstable();
    let mut actual: Vec<FaceId> = cube.cycles.ids().to_vec();
    actual.sort_unstable();
    assert_eq!(actual, expected, "survivors keep their face ids");
}

#[test]
fn expand_cube() {
    let mut cube = Shape::prism(4);
    assert_eq!(cube.order(), 8);
    assert_eq!(cube.edges().count(), 12);
    assert_eq!(cube.cycles.len(), 6);

    cube.expand();

    // Rhombicuboctahedron: V=24, E=48, F=26 (6 squares + 8 triangles + 12 squares).
    assert_eq!(cube.order(), 24, "vertex count");
    assert_eq!(cube.edges().count(), 48, "edge count");
    assert_eq!(cube.cycles.len(), 26, "face count");
    // Euler characteristic.
    assert_eq!(
        cube.order() as i64 - cube.edges().count() as i64 + cube.cycles.len() as i64,
        2
    );
    // Every vertex has degree 4 in a cantellation.
    for v in cube.vertices() {
        assert_eq!(cube.degree(v), 4, "vertex {v} degree");
    }
    // Face multiset: triangles + quads only, in the expected counts.
    let tris = cube.cycles.iter().filter(|c| c.len() == 3).count();
    let quads = cube.cycles.iter().filter(|c| c.len() == 4).count();
    assert_eq!(tris, 8, "triangle faces");
    assert_eq!(quads, 18, "quad faces");
}

#[test]
fn truncate_cube() {
    let mut cube = Shape::prism(4);
    assert_eq!(cube.order(), 8);
    assert_eq!(cube.edges().count(), 12);
    assert_eq!(cube.cycles.len(), 6);

    cube.truncate();

    // Truncated cube: V=24, E=36, F=14 (8 triangles + 6 octagons).
    assert_eq!(cube.order(), 24, "vertex count");
    assert_eq!(cube.edges().count(), 36, "edge count");
    assert_eq!(cube.cycles.len(), 14, "face count");
    // Euler characteristic.
    assert_eq!(
        cube.order() as i64 - cube.edges().count() as i64 + cube.cycles.len() as i64,
        2
    );
    // Every vertex has degree 3 in a truncation.
    for v in cube.vertices() {
        assert_eq!(cube.degree(v), 3, "vertex {v} degree");
    }
    let tris = cube.cycles.iter().filter(|c| c.len() == 3).count();
    let octs = cube.cycles.iter().filter(|c| c.len() == 8).count();
    assert_eq!(tris, 8, "triangle faces");
    assert_eq!(octs, 6, "octagon faces");
}

#[test]
#[ignore]
fn split_vertex_contract() {
    let mut control = Distance::new(6);
    // Original outline
    control[[1, 2]] = 1;
    control[[2, 3]] = 1;
    control[[3, 1]] = 1;
    // Connections
    control[[0, 1]] = 1;
    control[[4, 2]] = 1;
    control[[5, 3]] = 1;
    // New face
    control[[0, 4]] = 1;
    control[[4, 5]] = 1;
    control[[5, 0]] = 1;
    let mut test = Shape::from(Distance::tetrahedron());
    let edges = test.split_vertex(0)[1..].to_vec();
    test.distance.contract_edges(edges);
    assert_eq!(Distance::tetrahedron(), test.distance);
}
