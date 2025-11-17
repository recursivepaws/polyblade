use super::*;

impl Shape {
    pub fn floyd(&mut self) {
        self.distance.floyd();
    }
}

#[test]
#[ignore]
fn split_vertex_contract() {
    let mut control = Distance::new(6);
    // Original outline
    control[[1, 2]] = 1;
    control[[2, 3]] = 1;
    control[[3, 1]] = 1;
    // Connections
    control[[0, 1]] = 1;
    control[[4, 2]] = 1;
    control[[5, 3]] = 1;
    // New face
    control[[0, 4]] = 1;
    control[[4, 5]] = 1;
    control[[5, 0]] = 1;
    let mut test = Shape::from(Distance::tetrahedron());
    let edges = test.split_vertex(0)[1..].to_vec();
    test.distance.contract_edges(edges);
    assert_eq!(Distance::tetrahedron(), test.distance);
}
