use std::collections::HashSet;

mod control;
mod dfs;
mod lr_state;

use super::{Distance, Edge, VertexId};
use crate::try_control;

use control::*;
use dfs::*;
use lr_state::*;

#[cfg(test)]
mod dfs_test;
#[cfg(test)]
mod test;

// Filter edges by key and sort by nesting depth
// This allows us to ignore edges which are not tree or back edges,
// meaning we can skip it because it's going the wrong direction.
fn remaining_edges(w: VertexId, lr_state: &LRState) -> Vec<Edge> {
    let mut edges: Vec<Edge> = lr_state
        .graph
        .neighbors(w)
        .into_iter()
        .filter_map(|v| {
            let e: Edge = [v, w].into();
            lr_state.low_point.contains_key(&e).then_some(e)
        })
        .collect();
    edges.sort_by_key(|edge| lr_state.nesting_depth[edge]);
    // Remove parallel edges, which have no impact on planarity
    edges.dedup();
    edges
}

/// Visits the DFS - oriented tree that we have pre-computed
/// and stored in ``lr_state``. We traverse the edges of
/// a node in nesting depth order. Events are emitted at points
/// of interest and should be handled by ``visitor``.
fn lr_visit_ordered_dfs_tree(lr_state: &mut LRState, v: VertexId) -> Result<(), NonPlanar> {
    let mut stack: Vec<(VertexId, Vec<Edge>)> = vec![(v, remaining_edges(v, lr_state))];

    println!("stack: {stack:?}");

    while let Some(elem) = stack.last_mut() {
        let v = elem.0;
        let adjacent_edges = elem.1.clone();
        let mut next = None;

        for edge in adjacent_edges {
            if Some(&edge) == lr_state.edge_parent.get(&edge.w()) {
                lr_state.testing_visitor(LRTestDfsEvent::TreeEdge(edge))?;
                next = Some(edge.w());
                println!("newstack: {stack:?}");
                break;
            } else {
                lr_state.testing_visitor(LRTestDfsEvent::BackEdge(edge))?;
                lr_state.testing_visitor(LRTestDfsEvent::FinishEdge(edge))?;
            }
        }

        match next {
            Some(w) => {
                println!("remaining_edges: {:?}", remaining_edges(w, lr_state));
                stack.push((w, remaining_edges(w, lr_state)));
            }
            None => {
                stack.pop();
                println!("popped stack, finishing {v:?}");
                lr_state.testing_visitor(LRTestDfsEvent::Finish(v))?;

                if let Some(&edge) = lr_state.edge_parent.get(&v) {
                    println!("finishing edge.... from eparent {}, {}", edge.v(), edge.w());
                    lr_state.testing_visitor(LRTestDfsEvent::FinishEdge(edge))?;
                }
            }
        }
    }

    Ok(())
}
