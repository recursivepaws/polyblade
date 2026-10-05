//! Names the current polyhedron by its topology.
//!
//! The lookup key is a path-independent fingerprint, so `dtC` and `kC` land on
//! the same entry where the old Conway-string polydex would not have.
//!
//! The table is generated; see `src/polyhedron/shape/cycles/polydex_gen.rs` for the
//! recipe and `cargo test gen_polydex -- --ignored --nocapture` to reproduce it.

use crate::polyhedron::face::FaceTypeSignature;

pub struct Entry {
    pub key: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    /// Wikipedia article title, or empty when no article exists.
    pub wiki: &'static str,
}

/// Encodes a shape's vertex-degree multiset and face-signature multiset.
///
/// Both halves are needed. Degrees alone cannot tell the rhombicuboctahedron
/// from the elongated square gyrobicupola (J37) — both are `3.4.4.4` at every
/// vertex. Face signatures alone cannot tell the triakis tetrahedron from the
/// snub disphenoid (J84) — both are 12 all-triangle-neighbored triangles.
///
/// Format: `count@degree,…|count@sides:neighbor.neighbor.…;…`, e.g. the
/// cuboctahedron is `12@4|8@3:4.4.4;6@4:3.3.3.3`. Neighbors keep their dots so
/// that a 10-gon neighbour cannot be read as a 1 next to a 0.
pub fn key(degrees: &[usize], faces: &[FaceTypeSignature]) -> String {
    let mut degrees = degrees.to_vec();
    degrees.sort_unstable();
    let mut faces = faces.to_vec();
    faces.sort();
    format!(
        "{}|{}",
        run_length(&degrees, |d| d.to_string()).join(","),
        run_length(&faces, |f| format!(
            "{}:{}",
            f.side_count,
            f.neighbor_sides
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(".")
        ))
        .join(";")
    )
}

/// Collapses a sorted slice into `count@value` runs.
fn run_length<T: PartialEq>(sorted: &[T], show: impl Fn(&T) -> String) -> Vec<String> {
    let mut runs: Vec<String> = Vec::new();
    let mut i = 0;
    while i < sorted.len() {
        let run = sorted[i..].iter().take_while(|v| **v == sorted[i]).count();
        runs.push(format!("{run}@{}", show(&sorted[i])));
        i += run;
    }
    runs
}

pub fn lookup(key: &str) -> Option<&'static Entry> {
    TABLE.iter().find(|e| e.key == key)
}

#[rustfmt::skip]
pub const TABLE: &[Entry] = &[
    Entry { key: "4@3|4@3:3.3.3", name: "Tetrahedron", category: "Platonic", wiki: "Tetrahedron" },
    Entry { key: "8@3|6@4:4.4.4.4", name: "Cube", category: "Platonic", wiki: "Cube" },
    Entry { key: "6@4|8@3:3.3.3", name: "Octahedron", category: "Platonic", wiki: "Octahedron" },
    Entry { key: "20@3|12@5:5.5.5.5.5", name: "Dodecahedron", category: "Platonic", wiki: "Dodecahedron" },
    Entry { key: "12@5|20@3:3.3.3", name: "Icosahedron", category: "Platonic", wiki: "Icosahedron" },
    Entry { key: "12@3|4@3:6.6.6;4@6:3.3.3.6.6.6", name: "Truncated tetrahedron", category: "Archimedean", wiki: "Truncated tetrahedron" },
    Entry { key: "12@4|8@3:4.4.4;6@4:3.3.3.3", name: "Cuboctahedron", category: "Archimedean", wiki: "Cuboctahedron" },
    Entry { key: "24@3|8@3:8.8.8;6@8:3.3.3.3.8.8.8.8", name: "Truncated cube", category: "Archimedean", wiki: "Truncated cube" },
    Entry { key: "24@3|6@4:6.6.6.6;8@6:4.4.4.6.6.6", name: "Truncated octahedron", category: "Archimedean", wiki: "Truncated octahedron" },
    Entry { key: "24@4|8@3:4.4.4;12@4:3.3.4.4;6@4:4.4.4.4", name: "Rhombicuboctahedron", category: "Archimedean", wiki: "Rhombicuboctahedron" },
    Entry { key: "48@3|12@4:6.6.8.8;8@6:4.4.4.8.8.8;6@8:4.4.4.4.6.6.6.6", name: "Truncated cuboctahedron", category: "Archimedean", wiki: "Truncated cuboctahedron" },
    Entry { key: "24@5|8@3:3.3.3;24@3:3.3.4;6@4:3.3.3.3", name: "Snub cube", category: "Archimedean", wiki: "Snub cube" },
    Entry { key: "30@4|20@3:5.5.5;12@5:3.3.3.3.3", name: "Icosidodecahedron", category: "Archimedean", wiki: "Icosidodecahedron" },
    Entry { key: "60@3|20@3:10.10.10;12@10:3.3.3.3.3.10.10.10.10.10", name: "Truncated dodecahedron", category: "Archimedean", wiki: "Truncated dodecahedron" },
    Entry { key: "60@3|12@5:6.6.6.6.6;20@6:5.5.5.6.6.6", name: "Truncated icosahedron", category: "Archimedean", wiki: "Truncated icosahedron" },
    Entry { key: "60@4|20@3:4.4.4;30@4:3.3.5.5;12@5:4.4.4.4.4", name: "Rhombicosidodecahedron", category: "Archimedean", wiki: "Rhombicosidodecahedron" },
    Entry { key: "120@3|30@4:6.6.10.10;20@6:4.4.4.10.10.10;12@10:4.4.4.4.4.6.6.6.6.6", name: "Truncated icosidodecahedron", category: "Archimedean", wiki: "Truncated icosidodecahedron" },
    Entry { key: "60@5|20@3:3.3.3;60@3:3.3.5;12@5:3.3.3.3.3", name: "Snub dodecahedron", category: "Archimedean", wiki: "Snub dodecahedron" },
    Entry { key: "4@3,4@6|12@3:3.3.3", name: "Triakis tetrahedron", category: "Catalan", wiki: "Triakis tetrahedron" },
    Entry { key: "8@3,6@4|12@4:4.4.4.4", name: "Rhombic dodecahedron", category: "Catalan", wiki: "Rhombic dodecahedron" },
    Entry { key: "8@3,6@8|24@3:3.3.3", name: "Triakis octahedron", category: "Catalan", wiki: "Triakis octahedron" },
    Entry { key: "6@4,8@6|24@3:3.3.3", name: "Tetrakis hexahedron", category: "Catalan", wiki: "Tetrakis hexahedron" },
    Entry { key: "8@3,18@4|24@4:4.4.4.4", name: "Deltoidal icositetrahedron", category: "Catalan", wiki: "Deltoidal icositetrahedron" },
    Entry { key: "12@4,8@6,6@8|48@3:3.3.3", name: "Disdyakis dodecahedron", category: "Catalan", wiki: "Disdyakis dodecahedron" },
    Entry { key: "32@3,6@4|24@5:5.5.5.5.5", name: "Pentagonal icositetrahedron", category: "Catalan", wiki: "Pentagonal icositetrahedron" },
    Entry { key: "20@3,12@5|30@4:4.4.4.4", name: "Rhombic triacontahedron", category: "Catalan", wiki: "Rhombic triacontahedron" },
    Entry { key: "20@3,12@10|60@3:3.3.3", name: "Triakis icosahedron", category: "Catalan", wiki: "Triakis icosahedron" },
    Entry { key: "12@5,20@6|60@3:3.3.3", name: "Pentakis dodecahedron", category: "Catalan", wiki: "Pentakis dodecahedron" },
    Entry { key: "20@3,30@4,12@5|60@4:4.4.4.4", name: "Deltoidal hexecontahedron", category: "Catalan", wiki: "Deltoidal hexecontahedron" },
    Entry { key: "30@4,20@6,12@10|120@3:3.3.3", name: "Disdyakis triacontahedron", category: "Catalan", wiki: "Disdyakis triacontahedron" },
    Entry { key: "80@3,12@5|60@5:5.5.5.5.5", name: "Pentagonal hexecontahedron", category: "Catalan", wiki: "Pentagonal hexecontahedron" },
    Entry { key: "6@3|2@3:4.4.4;3@4:3.3.4.4", name: "Triangular prism", category: "Prism", wiki: "Triangular prism" },
    Entry { key: "10@3|5@4:4.4.5.5;2@5:4.4.4.4.4", name: "Pentagonal prism", category: "Prism", wiki: "Pentagonal prism" },
    Entry { key: "12@3|6@4:4.4.6.6;2@6:4.4.4.4.4.4", name: "Hexagonal prism", category: "Prism", wiki: "Hexagonal prism" },
    Entry { key: "14@3|7@4:4.4.7.7;2@7:4.4.4.4.4.4.4", name: "Heptagonal prism", category: "Prism", wiki: "Heptagonal prism" },
    Entry { key: "16@3|8@4:4.4.8.8;2@8:4.4.4.4.4.4.4.4", name: "Octagonal prism", category: "Prism", wiki: "Octagonal prism" },
    Entry { key: "18@3|9@4:4.4.9.9;2@9:4.4.4.4.4.4.4.4.4", name: "Enneagonal prism", category: "Prism", wiki: "Enneagonal prism" },
    Entry { key: "20@3|10@4:4.4.10.10;2@10:4.4.4.4.4.4.4.4.4.4", name: "Decagonal prism", category: "Prism", wiki: "Decagonal prism" },
    Entry { key: "22@3|11@4:4.4.11.11;2@11:4.4.4.4.4.4.4.4.4.4.4", name: "Hendecagonal prism", category: "Prism", wiki: "Hendecagonal prism" },
    Entry { key: "24@3|12@4:4.4.12.12;2@12:4.4.4.4.4.4.4.4.4.4.4.4", name: "Dodecagonal prism", category: "Prism", wiki: "Dodecagonal prism" },
    Entry { key: "8@4|8@3:3.3.4;2@4:3.3.3.3", name: "Square antiprism", category: "Antiprism", wiki: "Square antiprism" },
    Entry { key: "10@4|10@3:3.3.5;2@5:3.3.3.3.3", name: "Pentagonal antiprism", category: "Antiprism", wiki: "Pentagonal antiprism" },
    Entry { key: "12@4|12@3:3.3.6;2@6:3.3.3.3.3.3", name: "Hexagonal antiprism", category: "Antiprism", wiki: "Hexagonal antiprism" },
    Entry { key: "14@4|14@3:3.3.7;2@7:3.3.3.3.3.3.3", name: "Heptagonal antiprism", category: "Antiprism", wiki: "Heptagonal antiprism" },
    Entry { key: "16@4|16@3:3.3.8;2@8:3.3.3.3.3.3.3.3", name: "Octagonal antiprism", category: "Antiprism", wiki: "Octagonal antiprism" },
    Entry { key: "18@4|18@3:3.3.9;2@9:3.3.3.3.3.3.3.3.3", name: "Enneagonal antiprism", category: "Antiprism", wiki: "Enneagonal antiprism" },
    Entry { key: "20@4|20@3:3.3.10;2@10:3.3.3.3.3.3.3.3.3.3", name: "Decagonal antiprism", category: "Antiprism", wiki: "Decagonal antiprism" },
    Entry { key: "22@4|22@3:3.3.11;2@11:3.3.3.3.3.3.3.3.3.3.3", name: "Hendecagonal antiprism", category: "Antiprism", wiki: "" },
    Entry { key: "24@4|24@3:3.3.12;2@12:3.3.3.3.3.3.3.3.3.3.3.3", name: "Dodecagonal antiprism", category: "Antiprism", wiki: "" },
    Entry { key: "4@3,1@4|4@3:3.3.4;1@4:3.3.3.3", name: "Square pyramid", category: "Johnson J1", wiki: "Square pyramid" },
    Entry { key: "5@3,1@5|5@3:3.3.5;1@5:3.3.3.3.3", name: "Pentagonal pyramid", category: "Johnson J2", wiki: "Pentagonal pyramid" },
    Entry { key: "6@3,3@4|1@3:4.4.4;3@3:4.4.6;3@4:3.3.3.6;1@6:3.3.3.4.4.4", name: "Triangular cupola", category: "Johnson J3", wiki: "Triangular cupola" },
    Entry { key: "8@3,4@4|4@3:4.4.8;4@4:3.3.4.8;1@4:4.4.4.4;1@8:3.3.3.3.4.4.4.4", name: "Square cupola", category: "Johnson J4", wiki: "Square cupola" },
    Entry { key: "10@3,5@4|5@3:4.4.10;5@4:3.3.5.10;1@5:4.4.4.4.4;1@10:3.3.3.3.3.4.4.4.4.4", name: "Pentagonal cupola", category: "Johnson J5", wiki: "Pentagonal cupola" },
    Entry { key: "10@3,10@4|5@3:5.5.5;5@3:5.5.10;1@5:3.3.3.3.3;5@5:3.3.3.3.10;1@10:3.3.3.3.3.5.5.5.5.5", name: "Pentagonal rotunda", category: "Johnson J6", wiki: "Pentagonal rotunda" },
    Entry { key: "4@3,3@4|3@3:3.3.4;1@3:4.4.4;3@4:3.3.4.4", name: "Elongated triangular pyramid", category: "Johnson J7", wiki: "Elongated triangular pyramid" },
    Entry { key: "4@3,5@4|4@3:3.3.4;4@4:3.4.4.4;1@4:4.4.4.4", name: "Elongated square pyramid", category: "Johnson J8", wiki: "Elongated square pyramid" },
    Entry { key: "5@3,5@4,1@5|5@3:3.3.4;5@4:3.4.4.5;1@5:4.4.4.4.4", name: "Elongated pentagonal pyramid", category: "Johnson J9", wiki: "Elongated pentagonal pyramid" },
    Entry { key: "5@4,4@5|8@3:3.3.3;4@3:3.3.4;1@4:3.3.3.3", name: "Gyroelongated square pyramid", category: "Johnson J10", wiki: "Gyroelongated square pyramid" },
    Entry { key: "5@4,6@5|10@3:3.3.3;5@3:3.3.5;1@5:3.3.3.3.3", name: "Gyroelongated pentagonal pyramid", category: "Johnson J11", wiki: "Gyroelongated pentagonal pyramid" },
    Entry { key: "2@3,3@4|6@3:3.3.3", name: "Triangular bipyramid", category: "Johnson J12", wiki: "Triangular bipyramid" },
    Entry { key: "5@4,2@5|10@3:3.3.3", name: "Pentagonal bipyramid", category: "Johnson J13", wiki: "Pentagonal bipyramid" },
    Entry { key: "2@3,6@4|6@3:3.3.4;3@4:3.3.4.4", name: "Elongated triangular bipyramid", category: "Johnson J14", wiki: "Elongated triangular bipyramid" },
    Entry { key: "10@4|8@3:3.3.4;4@4:3.3.4.4", name: "Elongated square bipyramid", category: "Johnson J15", wiki: "Elongated square bipyramid" },
    Entry { key: "10@4,2@5|10@3:3.3.4;5@4:3.3.4.4", name: "Elongated pentagonal bipyramid", category: "Johnson J16", wiki: "Elongated pentagonal bipyramid" },
    Entry { key: "2@4,8@5|16@3:3.3.3", name: "Gyroelongated square bipyramid", category: "Johnson J17", wiki: "Gyroelongated square bipyramid" },
    Entry { key: "6@3,9@4|4@3:4.4.4;3@4:3.3.3.4;3@4:3.4.4.6;3@4:4.4.4.6;1@6:4.4.4.4.4.4", name: "Elongated triangular cupola", category: "Johnson J18", wiki: "Elongated triangular cupola" },
    Entry { key: "8@3,12@4|4@3:4.4.4;4@4:3.3.4.4;4@4:3.4.4.8;1@4:4.4.4.4;4@4:4.4.4.8;1@8:4.4.4.4.4.4.4.4", name: "Elongated square cupola", category: "Johnson J19", wiki: "Elongated square cupola" },
    Entry { key: "10@3,15@4|5@3:4.4.4;5@4:3.3.4.5;5@4:3.4.4.10;5@4:4.4.4.10;1@5:4.4.4.4.4;1@10:4.4.4.4.4.4.4.4.4.4", name: "Elongated pentagonal cupola", category: "Johnson J20", wiki: "Elongated pentagonal cupola" },
    Entry { key: "10@3,20@4|5@3:4.5.5;5@3:5.5.5;5@4:3.4.4.10;5@4:4.4.5.10;1@5:3.3.3.3.3;5@5:3.3.3.3.4;1@10:4.4.4.4.4.4.4.4.4.4", name: "Elongated pentagonal rotunda", category: "Johnson J21", wiki: "Elongated pentagonal rotunda" },
    Entry { key: "9@4,6@5|3@3:3.3.3;3@3:3.3.4;6@3:3.3.6;3@3:3.4.4;1@3:4.4.4;3@4:3.3.3.3;1@6:3.3.3.3.3.3", name: "Gyroelongated triangular cupola", category: "Johnson J22", wiki: "Gyroelongated triangular cupola" },
    Entry { key: "12@4,8@5|4@3:3.3.3;4@3:3.3.4;8@3:3.3.8;4@3:3.4.4;4@4:3.3.3.4;1@4:4.4.4.4;1@8:3.3.3.3.3.3.3.3", name: "Gyroelongated square cupola", category: "Johnson J23", wiki: "Gyroelongated square cupola" },
    Entry { key: "15@4,10@5|5@3:3.3.3;5@3:3.3.4;10@3:3.3.10;5@3:3.4.4;5@4:3.3.3.5;1@5:4.4.4.4.4;1@10:3.3.3.3.3.3.3.3.3.3", name: "Gyroelongated pentagonal cupola", category: "Johnson J24", wiki: "Gyroelongated pentagonal cupola" },
    Entry { key: "20@4,10@5|5@3:3.3.3;5@3:3.3.5;10@3:3.3.10;5@3:3.5.5;5@3:5.5.5;6@5:3.3.3.3.3;1@10:3.3.3.3.3.3.3.3.3.3", name: "Gyroelongated pentagonal rotunda", category: "Johnson J25", wiki: "Gyroelongated pentagonal rotunda" },
    Entry { key: "4@3,4@4|4@3:4.4.4;4@4:3.3.3.4", name: "Gyrobifastigium", category: "Johnson J26", wiki: "Gyrobifastigium" },
    Entry { key: "12@4|6@3:3.4.4;2@3:4.4.4;6@4:3.3.3.4", name: "Triangular orthobicupola", category: "Johnson J27", wiki: "Triangular orthobicupola" },
    Entry { key: "16@4|8@3:3.4.4;8@4:3.3.4.4;2@4:4.4.4.4", name: "Square orthobicupola", category: "Johnson J28", wiki: "Square orthobicupola" },
    Entry { key: "16@4|8@3:4.4.4;8@4:3.3.3.4;2@4:4.4.4.4", name: "Square gyrobicupola", category: "Johnson J29", wiki: "Square gyrobicupola" },
    Entry { key: "20@4|10@3:3.4.4;10@4:3.3.4.5;2@5:4.4.4.4.4", name: "Pentagonal orthobicupola", category: "Johnson J30", wiki: "Pentagonal orthobicupola" },
    Entry { key: "20@4|10@3:4.4.4;10@4:3.3.3.5;2@5:4.4.4.4.4", name: "Pentagonal gyrobicupola", category: "Johnson J31", wiki: "Pentagonal gyrobicupola" },
    Entry { key: "25@4|5@3:4.4.5;5@3:4.5.5;5@3:5.5.5;5@4:3.3.3.5;6@5:3.3.3.3.3;1@5:4.4.4.4.4", name: "Pentagonal orthocupolarotunda", category: "Johnson J32", wiki: "Pentagonal orthocupolarotunda" },
    Entry { key: "25@4|5@3:3.4.4;5@3:3.5.5;5@3:5.5.5;5@4:3.3.5.5;1@5:3.3.3.3.3;5@5:3.3.3.3.4;1@5:4.4.4.4.4", name: "Pentagonal gyrocupolarotunda", category: "Johnson J33", wiki: "Pentagonal gyrocupolarotunda" },
    Entry { key: "30@4|10@3:3.5.5;10@3:5.5.5;2@5:3.3.3.3.3;10@5:3.3.3.3.5", name: "Pentagonal orthobirotunda", category: "Johnson J34", wiki: "Pentagonal orthobirotunda" },
    Entry { key: "18@4|8@3:4.4.4;6@4:3.3.3.4;3@4:3.3.4.4;3@4:4.4.4.4", name: "Elongated triangular orthobicupola", category: "Johnson J35", wiki: "Elongated triangular orthobicupola" },
    Entry { key: "18@4|8@3:4.4.4;6@4:3.3.3.4;6@4:3.4.4.4", name: "Elongated triangular gyrobicupola", category: "Johnson J36", wiki: "Elongated triangular gyrobicupola" },
    Entry { key: "24@4|8@3:4.4.4;8@4:3.3.4.4;8@4:3.4.4.4;2@4:4.4.4.4", name: "Elongated square gyrobicupola", category: "Johnson J37", wiki: "Elongated square gyrobicupola" },
    Entry { key: "30@4|10@3:4.4.4;5@4:3.3.4.4;10@4:3.3.4.5;5@4:4.4.4.4;2@5:4.4.4.4.4", name: "Elongated pentagonal orthobicupola", category: "Johnson J38", wiki: "Elongated pentagonal orthobicupola" },
    Entry { key: "30@4|10@3:4.4.4;10@4:3.3.4.5;10@4:3.4.4.4;2@5:4.4.4.4.4", name: "Elongated pentagonal gyrobicupola", category: "Johnson J39", wiki: "Elongated pentagonal gyrobicupola" },
    Entry { key: "35@4|5@3:4.4.4;5@3:4.5.5;5@3:5.5.5;5@4:3.3.4.5;5@4:3.4.4.4;5@4:3.4.4.5;1@5:3.3.3.3.3;5@5:3.3.3.3.4;1@5:4.4.4.4.4", name: "Elongated pentagonal orthocupolarotunda", category: "Johnson J40", wiki: "Elongated pentagonal orthocupolarotunda" },
    Entry { key: "35@4|5@3:4.4.4;5@3:4.5.5;5@3:5.5.5;5@4:3.3.4.4;5@4:3.3.4.5;5@4:4.4.4.5;1@5:3.3.3.3.3;5@5:3.3.3.3.4;1@5:4.4.4.4.4", name: "Elongated pentagonal gyrocupolarotunda", category: "Johnson J41", wiki: "Elongated pentagonal gyrocupolarotunda" },
    Entry { key: "40@4|10@3:4.5.5;10@3:5.5.5;5@4:3.3.4.4;5@4:4.4.5.5;2@5:3.3.3.3.3;10@5:3.3.3.3.4", name: "Elongated pentagonal orthobirotunda", category: "Johnson J42", wiki: "Elongated pentagonal orthobirotunda" },
    Entry { key: "40@4|10@3:4.5.5;10@3:5.5.5;10@4:3.4.4.5;2@5:3.3.3.3.3;10@5:3.3.3.3.4", name: "Elongated pentagonal gyrobirotunda", category: "Johnson J43", wiki: "Elongated pentagonal gyrobirotunda" },
    Entry { key: "6@4,12@5|6@3:3.3.3;6@3:3.3.4;6@3:3.4.4;2@3:4.4.4;6@4:3.3.3.3", name: "Gyroelongated triangular bicupola", category: "Johnson J44", wiki: "Gyroelongated triangular bicupola" },
    Entry { key: "8@4,16@5|8@3:3.3.3;8@3:3.3.4;8@3:3.4.4;8@4:3.3.3.4;2@4:4.4.4.4", name: "Gyroelongated square bicupola", category: "Johnson J45", wiki: "Gyroelongated square bicupola" },
    Entry { key: "10@4,20@5|10@3:3.3.3;10@3:3.3.4;10@3:3.4.4;10@4:3.3.3.5;2@5:4.4.4.4.4", name: "Gyroelongated pentagonal bicupola", category: "Johnson J46", wiki: "Gyroelongated pentagonal bicupola" },
    Entry { key: "15@4,20@5|10@3:3.3.3;5@3:3.3.4;5@3:3.3.5;5@3:3.4.4;5@3:3.5.5;5@3:5.5.5;5@4:3.3.3.5;6@5:3.3.3.3.3;1@5:4.4.4.4.4", name: "Gyroelongated pentagonal cupolarotunda", category: "Johnson J47", wiki: "Gyroelongated pentagonal cupolarotunda" },
    Entry { key: "20@4,20@5|10@3:3.3.3;10@3:3.3.5;10@3:3.5.5;10@3:5.5.5;12@5:3.3.3.3.3", name: "Gyroelongated pentagonal birotunda", category: "Johnson J48", wiki: "Gyroelongated pentagonal birotunda" },
    Entry { key: "2@3,5@4|2@3:3.3.3;2@3:3.3.4;2@3:3.4.4;2@4:3.3.3.4", name: "Augmented triangular prism", category: "Johnson J49", wiki: "Augmented triangular prism" },
    Entry { key: "6@4,2@5|6@3:3.3.3;4@3:3.3.4;1@4:3.3.3.3", name: "Biaugmented triangular prism", category: "Johnson J50", wiki: "Biaugmented triangular prism" },
    Entry { key: "3@4,6@5|14@3:3.3.3", name: "Triaugmented triangular prism", category: "Johnson J51", wiki: "Triaugmented triangular prism" },
    Entry { key: "6@3,5@4|2@3:3.3.4;2@3:3.3.5;2@4:3.4.5.5;2@4:4.4.5.5;2@5:3.4.4.4.4", name: "Augmented pentagonal prism", category: "Johnson J52", wiki: "Augmented pentagonal prism" },
    Entry { key: "2@3,10@4|4@3:3.3.4;4@3:3.3.5;1@4:3.3.5.5;2@4:3.4.5.5;2@5:3.3.4.4.4", name: "Biaugmented pentagonal prism", category: "Johnson J53", wiki: "Biaugmented pentagonal prism" },
    Entry { key: "8@3,5@4|2@3:3.3.4;2@3:3.3.6;2@4:3.4.6.6;3@4:4.4.6.6;2@6:3.4.4.4.4.4", name: "Augmented hexagonal prism", category: "Johnson J54", wiki: "Augmented hexagonal prism" },
    Entry { key: "4@3,10@4|4@3:3.3.4;4@3:3.3.6;4@4:3.4.6.6;2@6:3.3.4.4.4.4", name: "Parabiaugmented hexagonal prism", category: "Johnson J55", wiki: "Parabiaugmented hexagonal prism" },
    Entry { key: "4@3,10@4|4@3:3.3.4;4@3:3.3.6;1@4:3.3.6.6;2@4:3.4.6.6;1@4:4.4.6.6;2@6:3.3.4.4.4.4", name: "Metabiaugmented hexagonal prism", category: "Johnson J56", wiki: "Metabiaugmented hexagonal prism" },
    Entry { key: "15@4|6@3:3.3.4;6@3:3.3.6;3@4:3.3.6.6;2@6:3.3.3.4.4.4", name: "Triaugmented hexagonal prism", category: "Johnson J57", wiki: "Triaugmented hexagonal prism" },
    Entry { key: "15@3,5@4,1@5|5@3:3.3.5;5@5:3.5.5.5.5;6@5:5.5.5.5.5", name: "Augmented dodecahedron", category: "Johnson J58", wiki: "Augmented dodecahedron" },
    Entry { key: "10@3,10@4,2@5|10@3:3.3.5;10@5:3.5.5.5.5", name: "Parabiaugmented dodecahedron", category: "Johnson J59", wiki: "Parabiaugmented dodecahedron" },
    Entry { key: "10@3,10@4,2@5|10@3:3.3.5;2@5:3.3.5.5.5;6@5:3.5.5.5.5;2@5:5.5.5.5.5", name: "Metabiaugmented dodecahedron", category: "Johnson J60", wiki: "Metabiaugmented dodecahedron" },
    Entry { key: "5@3,15@4,3@5|15@3:3.3.5;6@5:3.3.5.5.5;3@5:3.5.5.5.5", name: "Triaugmented dodecahedron", category: "Johnson J61", wiki: "Triaugmented dodecahedron" },
    Entry { key: "2@3,6@4,2@5|4@3:3.3.3;4@3:3.3.5;2@3:3.5.5;2@5:3.3.3.3.5", name: "Metabidiminished icosahedron", category: "Johnson J62", wiki: "Metabidiminished icosahedron" },
    Entry { key: "6@3,3@4|1@3:3.3.3;3@3:3.5.5;1@3:5.5.5;3@5:3.3.3.5.5", name: "Tridiminished icosahedron", category: "Johnson J63", wiki: "Tridiminished icosahedron" },
    Entry { key: "4@3,6@4|1@3:3.3.3;3@3:3.3.5;3@3:3.5.5;3@5:3.3.3.5.5", name: "Augmented tridiminished icosahedron", category: "Johnson J64", wiki: "Augmented tridiminished icosahedron" },
    Entry { key: "6@3,9@4|1@3:4.4.4;3@3:4.4.6;3@3:4.6.6;1@3:6.6.6;3@4:3.3.3.3;3@6:3.3.3.3.6.6", name: "Augmented truncated tetrahedron", category: "Johnson J65", wiki: "Augmented truncated tetrahedron" },
    Entry { key: "16@3,12@4|4@3:4.4.8;4@3:4.8.8;4@3:8.8.8;4@4:3.3.3.4;1@4:4.4.4.4;4@8:3.3.3.3.3.8.8.8;1@8:3.3.3.3.8.8.8.8", name: "Augmented truncated cube", category: "Johnson J66", wiki: "Augmented truncated cube" },
    Entry { key: "8@3,24@4|8@3:4.4.8;8@3:4.8.8;8@4:3.3.3.4;2@4:4.4.4.4;4@8:3.3.3.3.3.3.8.8", name: "Biaugmented truncated cube", category: "Johnson J67", wiki: "Biaugmented truncated cube" },
    Entry { key: "50@3,15@4|5@3:4.4.10;5@3:4.10.10;15@3:10.10.10;5@4:3.3.3.5;1@5:4.4.4.4.4;5@10:3.3.3.3.3.3.10.10.10.10;6@10:3.3.3.3.3.10.10.10.10.10", name: "Augmented truncated dodecahedron", category: "Johnson J68", wiki: "Augmented truncated dodecahedron" },
    Entry { key: "40@3,30@4|10@3:4.4.10;10@3:4.10.10;10@3:10.10.10;10@4:3.3.3.5;2@5:4.4.4.4.4;10@10:3.3.3.3.3.3.10.10.10.10", name: "Parabiaugmented truncated dodecahedron", category: "Johnson J69", wiki: "Parabiaugmented truncated dodecahedron" },
    Entry { key: "40@3,30@4|10@3:4.4.10;10@3:4.10.10;10@3:10.10.10;10@4:3.3.3.5;2@5:4.4.4.4.4;2@10:3.3.3.3.3.3.3.10.10.10;6@10:3.3.3.3.3.3.10.10.10.10;2@10:3.3.3.3.3.10.10.10.10.10", name: "Metabiaugmented truncated dodecahedron", category: "Johnson J70", wiki: "Metabiaugmented truncated dodecahedron" },
    Entry { key: "32@3,41@4,2@5|2@3:3.4.4;15@3:4.4.10;11@3:4.10.10;7@3:10.10.10;15@4:3.3.3.5;3@5:4.4.4.4.4;1@10:3.3.3.3.3.3.3.3.10.10;3@10:3.3.3.3.3.3.3.10.10.10;4@10:3.3.3.3.3.3.10.10.10.10;1@10:3.3.3.3.3.10.10.10.10.10", name: "Triaugmented truncated dodecahedron", category: "Johnson J71", wiki: "Triaugmented truncated dodecahedron" },
    Entry { key: "60@4|15@3:4.4.4;5@3:4.4.5;5@4:3.3.4.5;20@4:3.3.5.5;5@4:3.4.5.5;5@5:3.4.4.4.4;7@5:4.4.4.4.4", name: "Gyrate rhombicosidodecahedron", category: "Johnson J72", wiki: "Gyrate rhombicosidodecahedron" },
    Entry { key: "60@4|10@3:4.4.4;10@3:4.4.5;10@4:3.3.4.5;10@4:3.3.5.5;10@4:3.4.5.5;10@5:3.4.4.4.4;2@5:4.4.4.4.4", name: "Parabigyrate rhombicosidodecahedron", category: "Johnson J73", wiki: "Parabigyrate rhombicosidodecahedron" },
    Entry { key: "60@4|10@3:4.4.4;10@3:4.4.5;10@4:3.3.4.5;11@4:3.3.5.5;8@4:3.4.5.5;1@4:4.4.5.5;2@5:3.3.4.4.4;6@5:3.4.4.4.4;4@5:4.4.4.4.4", name: "Metabigyrate rhombicosidodecahedron", category: "Johnson J74", wiki: "Metabigyrate rhombicosidodecahedron" },
    Entry { key: "60@4|5@3:4.4.4;15@3:4.4.5;15@4:3.3.4.5;3@4:3.3.5.5;9@4:3.4.5.5;3@4:4.4.5.5;6@5:3.3.4.4.4;3@5:3.4.4.4.4;3@5:4.4.4.4.4", name: "Trigyrate rhombicosidodecahedron", category: "Johnson J75", wiki: "Trigyrate rhombicosidodecahedron" },
    Entry { key: "10@3,45@4|15@3:4.4.4;20@4:3.3.5.5;5@4:3.5.5.10;6@5:4.4.4.4.4;5@5:4.4.4.4.10;1@10:4.4.4.4.4.5.5.5.5.5", name: "Diminished rhombicosidodecahedron", category: "Johnson J76", wiki: "Diminished rhombicosidodecahedron" },
    Entry { key: "10@3,45@4|10@3:4.4.4;5@3:4.4.5;5@4:3.3.4.5;10@4:3.3.5.5;5@4:3.4.5.5;5@4:3.5.5.10;5@5:3.4.4.4.4;1@5:4.4.4.4.4;5@5:4.4.4.4.10;1@10:4.4.4.4.4.5.5.5.5.5", name: "Paragyrate diminished rhombicosidodecahedron", category: "Johnson J77", wiki: "Paragyrate diminished rhombicosidodecahedron" },
    Entry { key: "10@3,45@4|10@3:4.4.4;5@3:4.4.5;5@4:3.3.4.5;11@4:3.3.5.5;4@4:3.4.5.5;4@4:3.5.5.10;1@4:4.5.5.10;3@5:3.4.4.4.4;2@5:3.4.4.4.10;3@5:4.4.4.4.4;3@5:4.4.4.4.10;1@10:4.4.4.4.4.5.5.5.5.5", name: "Metagyrate diminished rhombicosidodecahedron", category: "Johnson J78", wiki: "Metagyrate diminished rhombicosidodecahedron" },
    Entry { key: "10@3,45@4|5@3:4.4.4;10@3:4.4.5;10@4:3.3.4.5;3@4:3.3.5.5;6@4:3.4.5.5;3@4:3.5.5.10;1@4:4.4.5.5;2@4:4.5.5.10;2@5:3.3.4.4.4;2@5:3.4.4.4.4;4@5:3.4.4.4.10;2@5:4.4.4.4.4;1@5:4.4.4.4.10;1@10:4.4.4.4.4.5.5.5.5.5", name: "Bigyrate diminished rhombicosidodecahedron", category: "Johnson J79", wiki: "Bigyrate diminished rhombicosidodecahedron" },
    Entry { key: "20@3,30@4|10@3:4.4.4;10@4:3.3.5.5;10@4:3.5.5.10;10@5:4.4.4.4.10;2@10:4.4.4.4.4.5.5.5.5.5", name: "Parabidiminished rhombicosidodecahedron", category: "Johnson J80", wiki: "Parabidiminished rhombicosidodecahedron" },
    Entry { key: "20@3,30@4|10@3:4.4.4;11@4:3.3.5.5;8@4:3.5.5.10;1@4:5.5.10.10;2@5:4.4.4.4.4;6@5:4.4.4.4.10;2@5:4.4.4.10.10;2@10:4.4.4.4.4.5.5.5.5.5", name: "Metabidiminished rhombicosidodecahedron", category: "Johnson J81", wiki: "Metabidiminished rhombicosidodecahedron" },
    Entry { key: "20@3,30@4|5@3:4.4.4;5@3:4.4.5;5@4:3.3.4.5;3@4:3.3.5.5;3@4:3.4.5.5;6@4:3.5.5.10;2@4:4.5.5.10;1@4:5.5.10.10;1@5:3.4.4.4.4;4@5:3.4.4.4.10;1@5:4.4.4.4.4;2@5:4.4.4.4.10;2@5:4.4.4.10.10;2@10:4.4.4.4.4.5.5.5.5.5", name: "Gyrate bidiminished rhombicosidodecahedron", category: "Johnson J82", wiki: "Gyrate bidiminished rhombicosidodecahedron" },
    Entry { key: "30@3,15@4|5@3:4.4.4;3@4:3.3.5.5;9@4:3.5.5.10;3@4:5.5.10.10;3@5:4.4.4.4.10;6@5:4.4.4.10.10;3@10:4.4.4.4.4.5.5.5.5.5", name: "Tridiminished rhombicosidodecahedron", category: "Johnson J83", wiki: "Tridiminished rhombicosidodecahedron" },
    Entry { key: "4@4,4@5|12@3:3.3.3", name: "Snub disphenoid", category: "Johnson J84", wiki: "Snub disphenoid" },
    Entry { key: "16@5|16@3:3.3.3;8@3:3.3.4;2@4:3.3.3.3", name: "Snub square antiprism", category: "Johnson J85", wiki: "Snub square antiprism" },
    Entry { key: "6@4,4@5|6@3:3.3.3;6@3:3.3.4;2@4:3.3.3.4", name: "Sphenocorona", category: "Johnson J86", wiki: "Sphenocorona" },
    Entry { key: "3@4,8@5|12@3:3.3.3;4@3:3.3.4;1@4:3.3.3.3", name: "Augmented sphenocorona", category: "Johnson J87", wiki: "Augmented sphenocorona" },
    Entry { key: "4@4,8@5|10@3:3.3.3;6@3:3.3.4;2@4:3.3.3.4", name: "Sphenomegacorona", category: "Johnson J88", wiki: "Sphenomegacorona" },
    Entry { key: "4@4,10@5|10@3:3.3.3;8@3:3.3.4;2@4:3.3.3.4;1@4:3.3.4.4", name: "Hebesphenomegacorona", category: "Johnson J89", wiki: "Hebesphenomegacorona" },
    Entry { key: "4@4,12@5|8@3:3.3.3;12@3:3.3.4;4@4:3.3.3.4", name: "Disphenocingulum", category: "Johnson J90", wiki: "Disphenocingulum" },
    Entry { key: "4@3,10@4|8@3:4.5.5;2@4:3.3.3.3;4@5:3.3.3.3.5", name: "Bilunabirotunda", category: "Johnson J91", wiki: "Bilunabirotunda" },
    Entry { key: "18@4|3@3:3.3.6;6@3:3.4.5;3@3:4.5.5;1@3:5.5.5;3@4:3.3.3.6;3@5:3.3.3.3.3;1@6:3.3.3.4.4.4", name: "Triangular hebesphenorotunda", category: "Johnson J92", wiki: "Triangular hebesphenorotunda" },
];
