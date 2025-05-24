use super::Distance;

#[derive(Debug)]
pub struct DfsResult {
    pub discovery_time: Vec<Option<usize>>,
    pub lowpoint: Vec<Option<usize>>,
    pub finish_time: Vec<Option<usize>>,
    pub parent: Vec<Option<usize>>,
}

impl DfsResult {
    fn new(num_vertices: usize) -> Self {
        Self {
            discovery_time: vec![None; num_vertices],
            lowpoint: vec![None; num_vertices],
            finish_time: vec![None; num_vertices],
            parent: vec![None; num_vertices],
        }
    }
}

impl Distance {
    pub fn dfs(&self) -> DfsResult {
        let mut result = DfsResult::new(self.order());
        let mut time = 0;

        // Visit all vertices to handle disconnected components
        for start_vertex in 0..self.order() {
            if result.discovery_time[start_vertex].is_none() {
                self.dfs_visit(start_vertex, &mut result, &mut time);
            }
        }

        result
    }

    fn dfs_visit(&self, u: usize, result: &mut DfsResult, time: &mut usize) {
        // Initialize discovery time and lowpoint
        *time += 1;
        result.discovery_time[u] = Some(*time);
        result.lowpoint[u] = Some(*time);

        // Explore all adjacent vertices
        for v in self.neighbors(u) {
            match result.discovery_time[v] {
                None => {
                    // Tree edge: v is unvisited
                    result.parent[v] = Some(u);
                    self.dfs_visit(v, result, time);

                    // Update lowpoint after visiting child
                    let child_lowpoint = result.lowpoint[v].unwrap();
                    let current_lowpoint = result.lowpoint[u].unwrap();
                    result.lowpoint[u] = Some(current_lowpoint.min(child_lowpoint));
                }
                Some(v_discovery) => {
                    // Back edge or forward/cross edge
                    if result.finish_time[v].is_none() {
                        // Back edge: v is an ancestor in DFS tree
                        let current_lowpoint = result.lowpoint[u].unwrap();
                        result.lowpoint[u] = Some(current_lowpoint.min(v_discovery));
                    }
                    // Forward/cross edges don't affect lowpoint
                }
            }
        }

        // Set finish time
        *time += 1;
        result.finish_time[u] = Some(*time);
    }
}

#[cfg(test)]
mod tests {
    use crate::polyhedron::shape::Distance;

    #[test]
    fn test_simple_graph() {
        let mut graph = Distance::new(4);
        graph.connect([0, 1]);
        graph.connect([1, 2]);
        graph.connect([2, 3]);
        graph.connect([3, 1]); // Back edge creating a cycle

        let result = graph.dfs();

        // Verify that all vertices were visited
        for i in 0..4 {
            assert!(result.discovery_time[i].is_some());
            assert!(result.lowpoint[i].is_some());
            assert!(result.finish_time[i].is_some());
        }

        println!("Discovery times: {:?}", result.discovery_time);
        println!("Lowpoints: {:?}", result.lowpoint);
        println!("Finish times: {:?}", result.finish_time);
        println!("Parents: {:?}", result.parent);
    }

    #[test]
    fn test_disconnected_components() {
        let mut graph = Distance::new(5);
        graph.connect([0, 1]);
        graph.connect([1, 2]);
        graph.connect([3, 4]); // Separate component

        let result = graph.dfs();

        println!("Discovery times: {:?}", result.discovery_time);
        println!("Lowpoints: {:?}", result.lowpoint);
        println!("Finish times: {:?}", result.finish_time);
        println!("Parents: {:?}", result.parent);
        // All vertices should be visited despite being disconnected
        for i in 0..5 {
            assert!(result.discovery_time[i].is_some());
            assert!(result.lowpoint[i].is_some());
            assert!(result.finish_time[i].is_some());
        }
    }
}
