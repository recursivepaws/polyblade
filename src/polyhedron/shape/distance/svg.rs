use super::Distance;
use graphviz_rust::{cmd::Format, exec, parse, printer::PrinterContext};

const LAYOUT_PREFIX: &str = r#"
    graph G {
        node [
            penwidth=2 
            label="" 
            style=filled
            fillcolor=lightblue
            color=black
            shape=circle
            width=0.25
            fixedsize=true
            fontsize=10
        ];
        edge [penwidth=2];
        overlap="scale";
        layout="neato";
        normalize=0;
        bgcolor="transparent";
"#;

impl Distance {
    pub fn graphviz(&self) -> String {
        let mut layout = LAYOUT_PREFIX.to_string();

        // #[cfg(test)]
        for v in self.vertices() {
            layout.push_str(&format!("\tV{v} [label=\"{}\"];\n", v));
        }

        for [v, u] in self.edges() {
            layout.push_str(&format!("\tV{v} -- V{u};\n"));
        }

        layout.push('}');
        // println!("graphviz:\n{layout}");
        layout
    }

    pub fn svg(&self) -> Option<Vec<u8>> {
        let Ok(graph) = parse(&self.graphviz()) else {
            log::warn!("failed to parse Graphviz");
            return None;
        };
        exec(
            graph,
            &mut PrinterContext::default(),
            vec![Format::Svg.into()],
        )
        .ok()
    }

    // #[cfg(test)]
    #[allow(dead_code)]
    pub fn png(&self) -> Option<Vec<u8>> {
        let Ok(graph) = parse(&self.graphviz()) else {
            log::warn!("failed to parse Graphviz");
            panic!("[sdfkjasdlfkj]");
        };

        exec(
            graph,
            &mut PrinterContext::default(),
            vec![Format::Png.into()],
        )
        .ok()
    }
}
