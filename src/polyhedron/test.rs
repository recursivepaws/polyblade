use super::*;
use crate::render::message::PresetMessage::{self, *};
use std::collections::HashSet;
use std::fs::create_dir_all;
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
#[ignore]
fn ambo() {
    use PresetMessage::*;
    let prefix = "tests/ambo/";
    create_dir_all(prefix).unwrap();
    let mut polyhedron = Polyhedron::preset(&Pyramid(3));
    polyhedron.ambo_contract();
    let octahedron = Polyhedron::preset(&Octahedron);
    assert_eq!(polyhedron.shape, octahedron.shape);
}

/// Applies an Ambo synchronously (matching how `ambo`/`truncate_contract` above bypass the
/// tick/transaction queue, since spring-convergence timing isn't relevant here), bracketed with
/// the same ancestry snapshot + reconcile calls `process_transactions` performs at its two hook
/// points.
fn apply_ambo(polyhedron: &mut Polyhedron) {
    let old_face_ancestors: Vec<HashSet<u64>> = (0..polyhedron.shape.cycles.len())
        .map(|i| polyhedron.shape.face_ancestors(i))
        .collect();
    let old_face_colors = polyhedron.face_colors.clone();
    polyhedron.ambo_contract();
    polyhedron.reconcile_face_colors(&old_face_ancestors, &old_face_colors);
}

/// Every face sharing a `FaceTypeSignature` must share a color.
fn assert_uniform_colors_per_facetype(polyhedron: &Polyhedron) {
    let signatures = polyhedron.face_signatures();
    let mut seen: Vec<(FaceTypeSignature, usize)> = Vec::new();
    for (i, sig) in signatures.iter().enumerate() {
        let color = polyhedron.face_colors[i];
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
    polyhedron.face_colors[i]
}

/// Ambo-ing a cube twice ("aaC") exercises exactly the scenario facetype-level consensus is for:
/// the cuboctahedron's triangles keep an unchanged signature into the rhombicuboctahedron and
/// must keep their color; the cuboctahedron's squares gain a new signature (their neighbors
/// change from triangles to the new vertex-figure squares) but must still color-continue via
/// ancestry; and the new vertex-figure squares are a genuinely new facetype needing a color
/// distinct from both.
#[test]
fn ambo_twice_preserves_facetype_colors() {
    let mut polyhedron = Polyhedron::preset(&Prism(4)); // cube ("C")

    apply_ambo(&mut polyhedron); // -> cuboctahedron
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

    apply_ambo(&mut polyhedron); // -> rhombicuboctahedron
    assert_uniform_colors_per_facetype(&polyhedron);

    // Unchanged signature; must keep its color.
    assert_eq!(color_for_signature(&polyhedron, &triangle), triangle_color);

    // New signature (bordered by the new vertex-figure squares instead of triangles), but must
    // still color-continue from the old squares via ancestry.
    let shrunk_square = FaceTypeSignature {
        side_count: 4,
        neighbor_sides: vec![4, 4, 4, 4],
    };
    assert_eq!(
        color_for_signature(&polyhedron, &shrunk_square),
        square_color
    );

    // Genuinely new facetype: must be distinct from both persisting facetypes' colors.
    let vertex_figure_square = FaceTypeSignature {
        side_count: 4,
        neighbor_sides: vec![3, 3, 4, 4],
    };
    let vertex_figure_color = color_for_signature(&polyhedron, &vertex_figure_square);
    assert_ne!(vertex_figure_color, triangle_color);
    assert_ne!(vertex_figure_color, square_color);
}

/// Ambo-ing an octahedron ("aO") also produces a cuboctahedron (triangles + squares), just via a
/// different construction path than aC: `Polyhedron::octahedron()` is itself built by ambo'ing a
/// tetrahedron at construction time, so the octahedron's own vertices already carry ancestry
/// accumulated from that construction before the user ever applies an Ambo to it.
#[test]
fn ambo_octahedron_gives_distinct_facetype_colors() {
    let mut polyhedron = Polyhedron::preset(&Octahedron);
    apply_ambo(&mut polyhedron); // -> cuboctahedron
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
