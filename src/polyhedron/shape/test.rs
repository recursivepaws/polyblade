use super::*;

impl Shape {
    pub fn floyd(&mut self) {
        self.distance.floyd();
    }
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
