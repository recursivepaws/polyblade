use super::*;

impl Distance {
    fn dfs(&self, state: &mut LRState) {
        let time = &mut 0;
        let discovered = &mut HashSet::with_capacity(self.order());
        let finished = &mut HashSet::with_capacity(self.order());

        // DFS orientation phase
        for node in self.vertices() {
            // try_control!(
            dfs_visitor(self.clone(), node, state, discovered, finished, time);
            // ,
            //     unreachable!()
            // );
        }
    }

    pub fn is_planar(&self) -> bool {
        let state = &mut LRState::new(self);

        self.dfs(state);

        // L-R partition phase
        for v in state.roots.clone() {
            if lr_visit_ordered_dfs_tree(state, v).is_err() {
                return false;
            }
        }

        true
    }
}

fn dfs_visitor(
    graph: Distance,
    u: VertexId,
    state: &mut LRState,
    //visitor: &mut F,
    //state.lr_orientation_visitor(event)
    discovered: &mut HashSet<VertexId>,
    finished: &mut HashSet<VertexId>,
    time: &mut Time,
) {
    if !discovered.insert(u) {
        return;
    }
    time_post_inc(time);

    // try_control!(state.lr_orientation_visitor(DfsEvent::Discover(u)), {}, {});

    //try_control!(
    state.lr_orientation_visitor(DfsEvent::Discover(u));

    //, {}, {
    let mut stack: Vec<(VertexId, Vec<VertexId>)> = Vec::new();
    stack.push((u, graph.neighbors(u)));

    while let Some((u, neighbors)) = stack.last() {
        let mut next = None;
        for &v in neighbors {
            let edge: Edge = [*u, v].into();
            // is_visited
            if !discovered.contains(&v) {
                try_control!(
                    state.lr_orientation_visitor(DfsEvent::TreeEdge(edge)),
                    continue
                );
                discovered.insert(v);
                time_post_inc(time);
                try_control!(
                    state.lr_orientation_visitor(DfsEvent::Discover(v)),
                    continue
                );
                next = Some(v);
                break;
            } else if !finished.contains(&v) {
                try_control!(
                    state.lr_orientation_visitor(DfsEvent::BackEdge(edge)),
                    continue
                );
            } else {
                try_control!(
                    state.lr_orientation_visitor(DfsEvent::CrossForwardEdge(edge)),
                    continue
                );
            }
        }

        match next {
            Some(v) => stack.push((v, graph.neighbors(v))),
            None => {
                let first_finish = finished.insert(*u);
                debug_assert!(first_finish);
                time_post_inc(time);
                try_control!(
                    state.lr_orientation_visitor(DfsEvent::Finish(*u)),
                    panic!("Pruning on the `DfsEvent::Finish` is not supported!")
                );
                stack.pop();
            }
        };
    }

    //);

    //C::continuing()
}

fn time_post_inc(x: &mut Time) -> Time {
    let v = *x;
    *x += 1;
    v
}
