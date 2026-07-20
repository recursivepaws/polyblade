use super::*;
use PresetMessage::*;

impl Polyhedron {
    pub fn preset(preset: &PresetMessage) -> Polyhedron {
        use PresetMessage::*;
        let mut polyhedron = match preset {
            Octahedron => Self::octahedron(),
            Dodecahedron => todo!(),
            Icosahedron => Self::icosahedron(),
            _ => {
                let shape = match preset {
                    Prism(n) => Shape::prism(*n),
                    AntiPrism(n) => Shape::anti_prism(*n),
                    Pyramid(n) => Shape::pyramid(*n),
                    _ => todo!(),
                };

                let render = Render::new(shape.order());

                Polyhedron {
                    name: preset.to_string(),
                    shape,
                    render,
                    transactions: vec![],
                    face_colors: vec![],
                    next_color_slot: 0,
                }
            }
        };
        // Bootstrapping is really "reconciling from nothing" — reset ancestry to a fresh
        // singleton tag per vertex too, so a preset built out of real Conway-style mutations
        // (e.g. `octahedron()` internally ambos a tetrahedron) doesn't leak that construction's
        // ancestry into whatever the user does next (see `Distance::reset_ancestry`).
        polyhedron.shape.reset_ancestry();
        (polyhedron.face_colors, polyhedron.next_color_slot) = polyhedron.bootstrap_face_colors();
        polyhedron
    }

    fn octahedron() -> Polyhedron {
        let mut polyhedron = Polyhedron::preset(&Pyramid(3));
        polyhedron.ambo_contract();
        polyhedron
    }
    pub fn icosahedron() -> Polyhedron {
        let mut graph = Polyhedron::preset(&AntiPrism(5));
        graph.shape.kis(Some(5));
        graph.render.new_capacity(graph.shape.order());
        graph
    }
}
