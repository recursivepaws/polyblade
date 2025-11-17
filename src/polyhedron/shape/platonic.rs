use super::{Distance, Shape};

impl Shape {
    pub fn prism(n: usize) -> Shape {
        Shape::from(Distance::prism(n))
    }

    pub fn anti_prism(n: usize) -> Shape {
        Shape::from(Distance::anti_prism(n))
    }

    pub fn pyramid(n: usize) -> Shape {
        Shape::from(Distance::pyramid(n))
    }
}
