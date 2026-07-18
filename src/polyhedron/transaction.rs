use crate::render::message::ConwayMessage;
use std::time::Instant;

use super::VertexId;

#[derive(Debug, Clone)]
pub enum Transaction {
    Contraction(Vec<[VertexId; 2]>),
    #[allow(dead_code)]
    Release(Vec<[VertexId; 2]>),
    Conway(ConwayMessage),
    #[allow(dead_code)]
    ShortenName(usize),
    Name(char),
    Wait(Instant),
    #[allow(dead_code)]
    None,
}
