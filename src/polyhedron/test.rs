use super::*;
use crate::render::message::PresetMessage::*;
//

impl Polyhedron {}

use test_case::test_case;
#[test_case(Polyhedron::preset(&Pyramid(3)); "T")]
#[test_case(Polyhedron::preset(&Prism(4)); "C")]
#[test_case(Polyhedron::preset(&Octahedron); "O")]
// #[test_case(Polyhedron::preset(&Dodecahedron); "D")]
#[test_case(Polyhedron::preset(&Icosahedron); "I")]
// #[test_case({ let mut g = Polyhedron::preset(&Prism(4)); g.truncate(0); g} ; "tC")]
// #[test_case({ let mut g = Polyhedron::preset(&Octahedron); g.truncate(0); g} ; "tO")]
// #[test_case({ let mut g = Polyhedron::preset(&Dodecahedron); g.truncate(0); g} ; "tD")]
fn polytope_apsp(poly: Polyhedron) {
    let mut bfs = poly.clone();
    bfs.shape.recompute();
    let mut floyd = poly.clone();
    floyd.shape.floyd();
    assert_eq!(bfs.shape, poly.shape);
    assert_eq!(bfs.shape, floyd.shape);
}

#[test]
#[ignore]
fn truncate_contract() {
    let mut shape = Polyhedron::preset(&Pyramid(3));
    let edges = shape.truncate(0);
    shape.contract(edges);
    assert_eq!(Polyhedron::preset(&Pyramid(3)).shape, shape.shape);
}

#[test]
fn ambo() {
    // Ambo tetrahedron == octahedron; exercises the one-shot truncate + contraction.
    let mut polyhedron = Polyhedron::preset(&Pyramid(3));
    polyhedron.ambo_contract();
    let octahedron = Polyhedron::preset(&Octahedron);
    assert_eq!(polyhedron.shape, octahedron.shape);
}

#[test]
fn ambo_cube_gives_cuboctahedron() {
    // Ambo cube: V=12, E=24, F=14 (8 triangles + 6 squares).
    let mut polyhedron = Polyhedron::preset(&Prism(4));
    polyhedron.ambo_contract();
    assert_eq!(polyhedron.shape.order(), 12, "vertex count");
    assert_eq!(polyhedron.shape.edges().count(), 24, "edge count");
    assert_eq!(polyhedron.shape.cycles.len(), 14, "face count");
    assert_eq!(polyhedron.render.positions.len(), 12, "render stays in sync");
}

fn apply_ambo(polyhedron: &mut Polyhedron) {
    polyhedron.cache_faces();
    polyhedron.ambo_contract();
    polyhedron.reconcile_face_colors();
}

fn apply_expand(polyhedron: &mut Polyhedron) {
    polyhedron.cache_faces();
    polyhedron.expand();
    polyhedron.reconcile_face_colors();
}

fn apply_truncate(polyhedron: &mut Polyhedron) {
    polyhedron.cache_faces();
    polyhedron.truncate(0);
    polyhedron.reconcile_face_colors();
}

/// Every face sharing a `FaceTypeSignature` must share a color.
fn assert_uniform_colors_per_facetype(polyhedron: &Polyhedron) {
    let signatures = polyhedron.face_signatures();
    let mut seen: Vec<(FaceTypeSignature, usize)> = Vec::new();
    for (i, sig) in signatures.iter().enumerate() {
        let color = polyhedron.face_coloring.colors[i];
        match seen.iter().find(|(s, _)| s == sig) {
            Some((_, expected)) => {
                assert_eq!(
                    color, *expected,
                    "faces of signature {sig:?} have inconsistent colors"
                )
            }
            None => seen.push((sig.clone(), color)),
        }
    }
}

/// The current color for a specific facetype; panics if no face currently has that signature.
fn color_for_signature(polyhedron: &Polyhedron, target: &FaceTypeSignature) -> usize {
    let signatures = polyhedron.face_signatures();
    let i = signatures
        .iter()
        .position(|sig| sig == target)
        .unwrap_or_else(|| panic!("no face with signature {target:?}"));
    polyhedron.face_coloring.colors[i]
}

#[test]
fn ambo_twice_preserves_facetype_colors() {
    // cube ("C")
    let mut polyhedron = Polyhedron::preset(&Prism(4));

    // cuboctahedron ("aC")
    apply_ambo(&mut polyhedron);
    assert_uniform_colors_per_facetype(&polyhedron);

    let triangle = FaceTypeSignature {
        side_count: 3,
        neighbor_sides: vec![4, 4, 4],
    };
    let square = FaceTypeSignature {
        side_count: 4,
        neighbor_sides: vec![3, 3, 3, 3],
    };
    let triangle_color = color_for_signature(&polyhedron, &triangle);
    let square_color = color_for_signature(&polyhedron, &square);
    assert_ne!(triangle_color, square_color);

    // rhombicuboctahedron ("aaC")
    apply_ambo(&mut polyhedron);
    assert_uniform_colors_per_facetype(&polyhedron);

    // Unchanged signature; must keep its color.
    assert_eq!(color_for_signature(&polyhedron, &triangle), triangle_color);

    // New signature but must still color-continue from the old squares via ancestry.
    let shrunk_square = FaceTypeSignature {
        side_count: 4,
        neighbor_sides: vec![4, 4, 4, 4],
    };
    assert_eq!(
        color_for_signature(&polyhedron, &shrunk_square),
        square_color
    );

    // Genuinely new facetype must be distinct from both persisting facetypes' colors.
    let vertex_figure_square = FaceTypeSignature {
        side_count: 4,
        neighbor_sides: vec![3, 3, 4, 4],
    };
    let vertex_figure_color = color_for_signature(&polyhedron, &vertex_figure_square);
    assert_ne!(vertex_figure_color, triangle_color);
    assert_ne!(vertex_figure_color, square_color);
}

#[test]
fn expand_preserves_facetype_colors() {
    // cube ("C"); every square borders four squares.
    let mut polyhedron = Polyhedron::preset(&Prism(4));
    let square = FaceTypeSignature {
        side_count: 4,
        neighbor_sides: vec![4, 4, 4, 4],
    };
    let square_color = color_for_signature(&polyhedron, &square);

    // rhombicuboctahedron ("eC")
    apply_expand(&mut polyhedron);
    assert_uniform_colors_per_facetype(&polyhedron);

    // Original faces persist as squares bordered by squares; must keep their color via ancestry.
    assert_eq!(color_for_signature(&polyhedron, &square), square_color);

    // Genuinely new facetypes must be distinct from the persisting square.
    let vertex_figure = FaceTypeSignature {
        side_count: 3,
        neighbor_sides: vec![4, 4, 4],
    };
    let edge_quad = FaceTypeSignature {
        side_count: 4,
        neighbor_sides: vec![3, 3, 4, 4],
    };
    assert_ne!(color_for_signature(&polyhedron, &vertex_figure), square_color);
    assert_ne!(color_for_signature(&polyhedron, &edge_quad), square_color);
}

#[test]
fn truncate_preserves_facetype_colors() {
    // cube ("C"); every square borders four squares.
    let mut polyhedron = Polyhedron::preset(&Prism(4));
    let square = FaceTypeSignature {
        side_count: 4,
        neighbor_sides: vec![4, 4, 4, 4],
    };
    let square_color = color_for_signature(&polyhedron, &square);

    // truncated cube ("tC")
    apply_truncate(&mut polyhedron);
    assert_uniform_colors_per_facetype(&polyhedron);

    // Each original square becomes an octagon bordered by 4 triangles + 4 octagons;
    // it must keep the square's color via ancestry.
    let octagon = FaceTypeSignature {
        side_count: 8,
        neighbor_sides: vec![3, 3, 3, 3, 8, 8, 8, 8],
    };
    assert_eq!(color_for_signature(&polyhedron, &octagon), square_color);

    // Vertex-figure triangles are a genuinely new facetype; must differ.
    let triangle = FaceTypeSignature {
        side_count: 3,
        neighbor_sides: vec![8, 8, 8],
    };
    assert_ne!(color_for_signature(&polyhedron, &triangle), square_color);
}

#[test]
fn dual_cube_gives_octahedron() {
    // Dual = expand, then contract the returned face-figure edges.
    let mut polyhedron = Polyhedron::preset(&Prism(4));
    let edges = polyhedron.dual();
    polyhedron.contract(edges);

    // Octahedron: V=6, E=12, F=8, all triangles.
    assert_eq!(polyhedron.shape.order(), 6, "vertex count");
    assert_eq!(polyhedron.shape.edges().count(), 12, "edge count");
    assert_eq!(polyhedron.shape.cycles.len(), 8, "face count");
    for c in polyhedron.shape.cycles.iter() {
        assert_eq!(c.len(), 3, "all faces are triangles");
    }
    assert_eq!(polyhedron.render.positions.len(), 6, "render stays in sync");
}

#[test]
fn dual_twice_is_identity() {
    // dd == identity: cube -> octahedron -> cube.
    let mut polyhedron = Polyhedron::preset(&Prism(4));
    let edges = polyhedron.dual();
    polyhedron.contract(edges);
    let edges = polyhedron.dual();
    polyhedron.contract(edges);

    assert_eq!(polyhedron.shape.order(), 8, "vertex count");
    assert_eq!(polyhedron.shape.edges().count(), 12, "edge count");
    assert_eq!(polyhedron.shape.cycles.len(), 6, "face count");
    for c in polyhedron.shape.cycles.iter() {
        assert_eq!(c.len(), 4, "all faces are squares");
    }
}

#[test]
fn ambo_octahedron_gives_distinct_facetype_colors() {
    // octahedron ("O")
    let mut polyhedron = Polyhedron::preset(&Octahedron);

    // cuboctahedron ("aO")
    apply_ambo(&mut polyhedron);
    assert_uniform_colors_per_facetype(&polyhedron);

    let triangle = FaceTypeSignature {
        side_count: 3,
        neighbor_sides: vec![4, 4, 4],
    };
    let square = FaceTypeSignature {
        side_count: 4,
        neighbor_sides: vec![3, 3, 3, 3],
    };
    assert_ne!(
        color_for_signature(&polyhedron, &triangle),
        color_for_signature(&polyhedron, &square)
    );
}
