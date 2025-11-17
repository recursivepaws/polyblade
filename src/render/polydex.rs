use crate::{bones::PolyGraph, polyhedron::Polyhedron, render::message::PolybladeMessage, Instant};
use serde::{Deserialize, Serialize};

pub type Polydex = Vec<Entry>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Entry {
    pub conway: String,
    pub name: String,
    pub bowers: String,
    pub wiki: String,
}

pub struct InfoBox {
    pub conway: String,
    pub faces: usize,
    pub edges: usize,
    pub vertices: usize,
    name: Option<String>,
    bowers: Option<String>,
    wiki: Option<String>,
}

impl InfoBox {
    pub const UNKNOWN: &'static str = "Unknown";

    pub fn name(&self) -> String {
        self.name.clone().unwrap_or(Self::UNKNOWN.to_string())
    }

    pub fn bowers(&self) -> String {
        self.bowers.clone().unwrap_or(Self::UNKNOWN.to_string())
    }

    pub fn wiki_message(&self) -> PolybladeMessage {
        self.wiki
            .clone()
            .map(PolybladeMessage::OpenWiki)
            .unwrap_or(PolybladeMessage::Tick(Instant::now()))
    }
}

impl Polyhedron {
    pub fn polydex_entry(&self, polydex: &Polydex) -> InfoBox {
        let entry = polydex.iter().find(|entry| entry.conway == self.name);
        InfoBox {
            conway: self.name.clone(),
            faces: 1,
            edges: 1,
            vertices: 1,
            name: entry.map(|e| e.name.clone()),
            bowers: entry.map(|e| e.bowers.clone()),
            wiki: entry.map(|e| e.wiki.clone()),
        }
    }
}
