use super::*;
use crossbeam_channel::unbounded;
use PresetMessage::*;

impl Polyhedron {
    pub fn preset(preset: &PresetMessage) -> Polyhedron {
        use PresetMessage::*;
        let (s, r) = unbounded();
        let mut poly = match preset {
            Octahedron => Self::octahedron(),
            Dodecahedron => Self::dodecahedron(),
            Icosahedron => Self::icosahedron(),
            _ => {
                let mut shape = match preset {
                    Prism(n) => Shape::prism(*n),
                    AntiPrism(n) => Shape::anti_prism(*n),
                    Pyramid(n) => Shape::pyramid(*n),
                    _ => todo!(),
                };

                let mut render = Render::new(shape.order());

                shape.set_sender(s);
                render.set_receiver(r);

                Polyhedron {
                    name: preset.to_string(),
                    shape,
                    render,
                    transactions: vec![],
                }
            }
        };
        poly.shape.compute_graph_svg();
        poly
    }

    fn octahedron() -> Polyhedron {
        let mut polyhedron = Polyhedron::preset(&Pyramid(3));
        polyhedron.ambo_contract();
        polyhedron
    }

    fn dodecahedron() -> Polyhedron {
        let mut graph = Polyhedron::preset(&AntiPrism(5));
        graph.expand();
        graph.render.new_capacity(graph.shape.order());
        graph.name = "D".to_string();
        graph
    }

    fn icosahedron() -> Polyhedron {
        let mut graph = Polyhedron::preset(&AntiPrism(5));
        graph.shape.kis(Some(5));
        graph.render.new_capacity(graph.shape.order());
        graph.name = "I".to_string();
        graph
    }
}

impl Default for Polyhedron {
    fn default() -> Self {
        Self::preset(&Prism(4))
    }
}
