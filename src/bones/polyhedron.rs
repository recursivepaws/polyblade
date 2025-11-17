use crate::{
    bones::{Edge, PolyGraph, Transaction},
    render::message::ConwayMessage,
};
use std::time::{Duration, Instant};
use ultraviolet::{Lerp, Vec3};

const SPEED_DAMPENING: f32 = 0.92;

// Operations
impl PolyGraph {
    fn apply_spring_forces(&mut self, speed: f32, second: f32) {
        let diameter = *self.dist.values().max().unwrap_or(&1) as f32;
        let diameter_spring_length = self.edge_length * 2.0;
        let (edges, contracting): (std::collections::hash_set::Iter<Edge>, bool) =
            if let Some(Transaction::Contraction(edges)) = self.transactions.first() {
                (edges.iter(), true)
            } else {
                (self.springs.iter(), false)
            };

        for e in edges {
            let v = e.v();
            let u = e.u();
            let v_position = self.positions[&v];
            let u_position = self.positions[&u];
            let diff = v_position - u_position;
            let spring_length = diff.mag();
            if contracting {
                let f = ((self.edge_length / speed * second) * 10.0) / spring_length;
                *self.positions.entry(v).or_default() = v_position.lerp(u_position, f);
                *self.positions.entry(u).or_default() = u_position.lerp(v_position, f);
            } else {
                let target_length = diameter_spring_length * (self.dist[e] as f32 / diameter);
                let f = diff * (target_length - spring_length) / speed * second;
                *self.speeds.entry(v).or_default() = (self.speeds[&v] + f) * SPEED_DAMPENING;
                *self.speeds.entry(u).or_default() = (self.speeds[&u] - f) * SPEED_DAMPENING;
                *self.positions.entry(v).or_default() += self.speeds[&v];
                *self.positions.entry(u).or_default() += self.speeds[&u];
            }
        }
    }

    fn center(&mut self) {
        let shift =
            self.positions.values().fold(Vec3::zero(), |a, &b| a + b) / self.vertices.len() as f32;

        for (_, v) in self.positions.iter_mut() {
            *v -= shift;
        }
    }

    fn resize(&mut self, speed: f32, second: f32) {
        let mean_length = self.positions.values().map(|p| p.mag()).fold(0.0, f32::max);
        let distance = mean_length - 1.0;
        self.edge_length -= distance / speed * second;
    }

    pub fn update(&mut self, speed: f32, second: f32) {
        self.center();
        self.resize(speed, second);
        self.apply_spring_forces(speed, second);
        self.process_transactions(speed);
    }

    pub fn face_positions(&self, face_index: usize) -> Vec<Vec3> {
        self.cycles[face_index]
            .iter()
            .map(|v| self.positions[v])
            .collect()
    }

    pub fn face_centroid(&self, face_index: usize) -> Vec3 {
        // All vertices associated with this face
        let vertices: Vec<_> = self.face_positions(face_index);
        vertices.iter().fold(Vec3::zero(), |a, &b| a + b) / vertices.len() as f32
    }

    pub fn process_transactions(&mut self, speed: f32) {
        if let Some(transaction) = self.transactions.first().cloned() {
            use Transaction::*;
            match transaction {
                Contraction(edges) => {
                    if !edges
                        .iter()
                        .any(|e| (self.positions[&e.v()] - self.positions[&e.u()]).mag() > 0.02)
                    {
                        // Contract them in the graph
                        self.contract_edges(edges);
                        self.pst();
                        self.springs();
                        self.transactions.remove(0);
                    }
                }
                Release(edges) => {
                    for e in edges.into_iter() {
                        self.disconnect(e);
                    }
                    self.pst();
                    self.springs();
                    self.find_cycles();
                    self.transactions.remove(0);
                }
                Conway(conway) => {
                    self.transactions.remove(0);
                    use ConwayMessage::*;
                    use Transaction::*;
                    let new_transactions = match conway {
                        Chamfer => todo!(),
                        Dual => {
                            let edges = self.expand(false);
                            vec![
                                Wait(Instant::now() + Duration::from_millis((65.0 * speed) as u64)),
                                Contraction(edges),
                                Name('d'),
                            ]
                        }
                        Join => {
                            let edges = self.kis(Option::None);
                            vec![
                                //Wait(Instant::now() + Duration::from_secs(1)),
                                Release(edges),
                                Name('j'),
                            ]
                        }
                        Ambo => {
                            let edges = self.ambo();
                            vec![Contraction(edges), Name('a')]
                        }
                        Kis => {
                            self.kis(Option::None);
                            vec![Name('k')]
                        }
                        Truncate => {
                            self.truncate(Option::None);
                            vec![Name('t')]
                        }
                        Expand => {
                            self.expand(false);
                            vec![Name('e')]
                        }
                        Snub => {
                            self.expand(true);
                            vec![Name('s')]
                        }
                        Bevel => {
                            vec![
                                Conway(Truncate),
                                Wait(Instant::now() + Duration::from_millis(500)),
                                Conway(Ambo),
                                Name('b'),
                            ]
                        }
                    };
                    self.cycles.sort_by_key(|c| usize::MAX - c.len());
                    self.transactions = [new_transactions, self.transactions.clone()].concat();
                    self.pst();
                    self.springs();
                }
                Name(c) => {
                    if c == 'b' {
                        self.name = self.name[2..].to_string();
                    }
                    if c == 'd' && &self.name[0..1] == "d" {
                        self.name = self.name[1..].to_string();
                    } else {
                        self.name = format!("{c}{}", self.name);
                    }
                    self.transactions.remove(0);
                }
                ShortenName(n) => {
                    self.name = self.name[n..].to_string();
                    self.transactions.remove(0);
                }
                Wait(instant) => {
                    if Instant::now() > instant {
                        self.transactions.remove(0);
                    }
                }
                None => {}
            }
        }
    }
}
