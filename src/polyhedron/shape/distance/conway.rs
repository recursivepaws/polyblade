use super::Distance;
use crate::polyhedron::shape::Cycle;
use crate::polyhedron::VertexId;

impl Distance {
    pub(super) fn contract_edge(&mut self, [v, u]: [VertexId; 2]) {
        // Give u all the same connections as v
        for w in self.neighbors(v).into_iter() {
            self.connect([w, u]);
            self.disconnect([w, v]);
        }
        // Delete v
        self.delete(v);
    }

    pub fn contract_edges(&mut self, mut edges: Vec<[VertexId; 2]>) {
        while !edges.is_empty() {
            // Pop an edge
            let [w, x] = edges.remove(0);
            let v = w.max(x);
            let u = w.min(x);

            // Contract [v, u], deleting v
            self.contract_edge([v, u]);
            // Decrement the value of every vertex
            for [x, w] in &mut edges {
                if *x > v {
                    *x -= 1;
                }
                if *w > v {
                    *w -= 1;
                }
            }
        }
    }

    pub fn split_vertex(&mut self, v: VertexId, connections: Vec<VertexId>) -> Vec<[VertexId; 2]> {
        // Remove the vertex
        let new_cycle: Cycle = Cycle::from(
            vec![v]
                .into_iter()
                .chain((1..connections.len()).map(|_| self.insert()))
                .collect(),
        );

        for c in &connections {
            self.disconnect([v, *c]);
        }

        for i in 0..new_cycle.len() {
            self.connect([new_cycle[i], connections[i]]);
        }

        // track the edges that will compose the new face
        let mut new_edges = vec![];
        for i in 0..new_cycle.len() {
            let edge = [new_cycle[i], new_cycle[i + 1]];
            self.connect(edge);
            new_edges.push(edge);
        }

        new_edges
    }

    //
    // `j` join
    // `z` zip
    // `g` gyro
    // `m` meta = `kj`
    // `o` ortho = `jj`
    // `n` needle
    // `k` kis
}
