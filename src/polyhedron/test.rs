use super::*;
use crate::render::message::PresetMessage::*;
//

impl Polyhedron {}

use test_case::test_case;
#[test_case(Polyhedron::preset(&Pyramid(3)); "T")]
#[test_case(Polyhedron::preset(&Prism(4)); "C")]
#[test_case(Polyhedron::preset(&Octahedron); "O")]
#[test_case(Polyhedron::preset(&Dodecahedron); "D")]
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

/// Index of the first face with the given signature; panics if none matches.
fn signature_index(polyhedron: &Polyhedron, target: &FaceTypeSignature) -> usize {
    polyhedron
        .face_signatures()
        .iter()
        .position(|sig| sig == target)
        .unwrap_or_else(|| panic!("no face with signature {target:?}"))
}

/// The current color slot for a specific facetype.
fn color_for_signature(polyhedron: &Polyhedron, target: &FaceTypeSignature) -> usize {
    polyhedron.face_coloring.colors[signature_index(polyhedron, target)]
}

/// The rendered palette index (what the UI actually shows) for a specific facetype.
fn render_index_for_signature(polyhedron: &Polyhedron, target: &FaceTypeSignature) -> usize {
    polyhedron.face_coloring.render_indices[signature_index(polyhedron, target)]
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

/// The color slot of the first face with the given side count; panics if none matches.
fn color_by_side(polyhedron: &Polyhedron, side: usize) -> usize {
    let i = polyhedron
        .shape
        .cycles
        .iter()
        .position(|c| c.len() == side)
        .unwrap_or_else(|| panic!("no face with {side} sides"));
    polyhedron.face_coloring.colors[i]
}

#[test]
fn truncate_cuboctahedron_keeps_square_color_on_octagons() {
    // cube -> ambo -> cuboctahedron (6 squares + 8 triangles).
    let mut polyhedron = Polyhedron::preset(&Prism(4));
    apply_ambo(&mut polyhedron);
    let square = FaceTypeSignature {
        side_count: 4,
        neighbor_sides: vec![3, 3, 3, 3],
    };
    let square_color = color_for_signature(&polyhedron, &square);

    // truncate -> truncated cuboctahedron; each square becomes an octagon and must keep its color.
    apply_truncate(&mut polyhedron);
    assert_uniform_colors_per_facetype(&polyhedron);

    // The octagons descend from the squares, so they inherit the color; the new vertex-figure squares must not steal it.
    assert_eq!(color_by_side(&polyhedron, 8), square_color, "octagons keep the square color");
    assert_ne!(color_by_side(&polyhedron, 4), square_color, "new squares must not steal the square color");
}

#[test]
fn dodecahedron_is_well_formed() {
    // dual(antiprism 5) then truncate its two degree-5 apexes -> dodecahedron.
    let polyhedron = Polyhedron::preset(&Dodecahedron);
    assert_eq!(polyhedron.shape.order(), 20, "vertex count");
    assert_eq!(polyhedron.shape.edges().count(), 30, "edge count");
    assert_eq!(polyhedron.shape.cycles.len(), 12, "face count");
    for c in polyhedron.shape.cycles.iter() {
        assert_eq!(c.len(), 5, "all faces are pentagons");
    }
    for v in polyhedron.shape.vertices() {
        assert_eq!(polyhedron.shape.degree(v), 3, "vertex {v} degree");
    }
    assert_eq!(polyhedron.render.positions.len(), 20, "render stays in sync");
}

#[test]
fn dual_cube_gives_octahedron() {
    // Dual = expand, then contract the returned face-figure edges.
    let mut polyhedron = Polyhedron::preset(&Prism(4));
    let edges = polyhedron.begin_dual();
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
fn dual_preserves_triangle_color_continuity() {
    // Mirrors the Dual transaction: expand (cube -> rhombicuboctahedron), then
    // contract the face-figures (-> octahedron). The surviving vertex-figure
    // triangles must keep their color across the contraction.
    let mut polyhedron = Polyhedron::preset(&Prism(4));

    polyhedron.cache_faces();
    let edges = polyhedron.begin_dual();
    polyhedron.reconcile_face_colors();
    let triangle = FaceTypeSignature {
        side_count: 3,
        neighbor_sides: vec![4, 4, 4],
    };
    let pink_slot = color_for_signature(&polyhedron, &triangle);
    let pink_render = render_index_for_signature(&polyhedron, &triangle);

    polyhedron.cache_faces();
    polyhedron.contract(edges);
    polyhedron.reconcile_face_colors();
    assert_uniform_colors_per_facetype(&polyhedron);

    let octahedron_triangle = FaceTypeSignature {
        side_count: 3,
        neighbor_sides: vec![3, 3, 3],
    };
    // Both the color slot and — crucially — the rendered palette index must carry over,
    // since the UI shows `palette[render_index]`, not the slot.
    assert_eq!(
        color_for_signature(&polyhedron, &octahedron_triangle),
        pink_slot,
        "octahedron triangles keep the rhombicuboctahedron triangle color slot"
    );
    assert_eq!(
        render_index_for_signature(&polyhedron, &octahedron_triangle),
        pink_render,
        "octahedron triangles render the same palette color as before"
    );
}

#[test]
fn survivor_keeps_color_while_freed_colors_rotate_to_the_back() {
    // The tetrahedron is self-dual. Its surviving face color must never change, but the
    // transient facetypes created along the way should advance through the palette: a color
    // freed by a disappearing facetype goes to the back, so the next new facetype picks a
    // fresh entry rather than recycling the one just freed.
    let mut polyhedron = Polyhedron::preset(&Pyramid(3));
    polyhedron.face_coloring.set_palette_len(6);
    let triangle = FaceTypeSignature {
        side_count: 3,
        neighbor_sides: vec![3, 3, 3],
    };
    let square = FaceTypeSignature {
        side_count: 4,
        neighbor_sides: vec![3, 3, 3, 3],
    };
    let tetra_color = render_index_for_signature(&polyhedron, &triangle);

    // First dual: capture the intermediate cuboctahedron's square palette entry.
    polyhedron.cache_faces();
    let edges = polyhedron.begin_dual();
    polyhedron.reconcile_face_colors();
    let first_square = render_index_for_signature(&polyhedron, &square);
    polyhedron.cache_faces();
    polyhedron.contract(edges);
    polyhedron.reconcile_face_colors();
    assert_eq!(
        render_index_for_signature(&polyhedron, &triangle),
        tetra_color,
        "tetrahedron keeps its color after one dual"
    );

    // Second dual: the recreated square advances to a fresh palette entry (the freed one is
    // now at the back), and the surviving tetrahedron still holds its original color.
    polyhedron.cache_faces();
    let edges = polyhedron.begin_dual();
    polyhedron.reconcile_face_colors();
    let second_square = render_index_for_signature(&polyhedron, &square);
    assert_ne!(
        second_square, first_square,
        "recreated square advances instead of recycling the just-freed color"
    );
    assert_ne!(
        second_square, tetra_color,
        "recreated square never collides with the surviving facetype's color"
    );
    polyhedron.contract(edges);
    polyhedron.reconcile_face_colors();
    assert_eq!(
        render_index_for_signature(&polyhedron, &triangle),
        tetra_color,
        "tetrahedron keeps its color after a second dual"
    );
}

#[test]
fn dual_twice_is_identity() {
    // dd == identity: cube -> octahedron -> cube.
    let mut polyhedron = Polyhedron::preset(&Prism(4));
    let edges = polyhedron.begin_dual();
    polyhedron.contract(edges);
    let edges = polyhedron.begin_dual();
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
