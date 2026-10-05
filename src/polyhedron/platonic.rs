use super::*;
use PresetMessage::*;

impl Polyhedron {
    pub fn preset(preset: &PresetMessage) -> Polyhedron {
        use PresetMessage::*;
        let mut polyhedron = match preset {
            Octahedron => Self::octahedron(),
            Dodecahedron => Self::dodecahedron(),
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
                    name: String::new(),
                    shape,
                    render,
                    transactions: vec![],
                    face_coloring: FaceColoring::default(),
                }
            }
        };
        // Set last: the three solids built above come out of other presets, so they
        // would otherwise inherit the seed letter of whatever they were built from.
        polyhedron.name = preset.conway_seed();
        // Bootstrapping assigns fresh colors regardless of construction-time operations.
        polyhedron.bootstrap_face_colors();
        polyhedron
    }

    fn octahedron() -> Polyhedron {
        let mut polyhedron = Polyhedron::preset(&Pyramid(3));
        polyhedron.ambo_contract();
        polyhedron
    }

    pub fn dodecahedron() -> Polyhedron {
        let mut graph = Polyhedron::preset(&AntiPrism(5));
        graph.dual();
        graph.truncate(5);
        graph
    }

    pub fn icosahedron() -> Polyhedron {
        let mut graph = Polyhedron::preset(&AntiPrism(5));
        graph.shape.kis(Some(5));
        graph.render.new_capacity(graph.shape.order());
        graph
    }
}
