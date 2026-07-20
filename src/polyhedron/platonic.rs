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
                    face_coloring: FaceColoring::default(),
                }
            }
        };
        // Bootstrapping is "reconciling from nothing", so reset ancestry here too.
        // Otherwise e.g. octahedron's internal construction-time ambo leaks into the user's first op.
        polyhedron.shape.reset_ancestry();
        let (colors, next_color_slot) = polyhedron.bootstrap_face_colors();
        polyhedron.face_coloring.bootstrap(colors, next_color_slot);
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
