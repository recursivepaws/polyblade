use super::Distance;
use crate::polyhedron::VertexId;
use crate::polyhedron::shape::Cycle;

impl Distance {
    pub(super) fn contract_edge(&mut self, [v, u]: [VertexId; 2]) {
        // Give u all the same connections as v
        for w in self.neighbors(v).into_iter() {
            self.connect([w, u]);
            self.disconnect([w, v]);
        }
        // Delete v; u now represents both original vertices
        self.delete(v);
    }

    pub fn contract_edges(&mut self, edges: Vec<[VertexId; 2]>) {
        crate::polyhedron::contract_edge_indices(edges, |v, u| {
            self.contract_edge([v, u]);
        });
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
