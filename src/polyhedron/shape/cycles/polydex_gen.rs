//! Generator for `crate::polydex::TABLE`: `cargo test gen_polydex -- --ignored --nocapture`.
//! Lives in `cycles` to reuse the runtime's `orient_faces`, so keys match `Polyhedron::polydex_key` byte for byte.

use super::{neighbor_type_signatures, orient_faces};
use crate::polyhedron::Polyhedron;
use crate::polyhedron::face::FaceTypeSignature;
use crate::render::message::PresetMessage::*;
use std::collections::{BTreeSet, HashMap};

type Faces = Vec<Vec<usize>>;

const SOURCE: &str = include_str!("polydex_source.txt");

/// Parses `polydex_source.txt`, vendored from <https://netlib.sandia.gov/polyhedra/>, into `(index, name, faces)`.
/// `faces` is empty for the solids netlib only describes as a 2D net.
fn source() -> Vec<(usize, String, Faces)> {
    SOURCE
        .lines()
        .map(|line| {
            let mut fields = line.split('|');
            let index = fields.next().unwrap().parse().unwrap();
            let name = fields.next().unwrap().to_string();
            let faces = fields
                .next()
                .unwrap()
                .split_terminator(';')
                .map(|face| face.split(',').map(|v| v.parse().unwrap()).collect())
                .collect();
            (index, name, faces)
        })
        .collect()
}

/// Compacts vertex ids down to `0..n`, preserving relative order.
fn renumber(faces: &[Vec<usize>]) -> Faces {
    let used: Vec<usize> = faces
        .iter()
        .flatten()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    faces
        .iter()
        .map(|f| f.iter().map(|v| used.binary_search(v).unwrap()).collect())
        .collect()
}

/// Undirected edge set, with how many faces border each edge.
fn edge_counts(faces: &[Vec<usize>]) -> HashMap<[usize; 2], usize> {
    let mut counts = HashMap::new();
    for face in faces {
        for k in 0..face.len() {
            let (a, b) = (face[k], face[(k + 1) % face.len()]);
            *counts.entry([a.min(b), a.max(b)]).or_insert(0) += 1;
        }
    }
    counts
}

/// A closed surface of genus 0, or a panic naming what is wrong with it.
fn assert_sphere(faces: &[Vec<usize>], label: &str) {
    let counts = edge_counts(faces);
    let open: Vec<_> = counts.iter().filter(|(_, n)| **n != 2).collect();
    assert!(
        open.is_empty(),
        "{label}: {} edges not shared by exactly two faces: {open:?}",
        open.len()
    );
    let (v, e, f) = (
        faces.iter().flatten().collect::<BTreeSet<_>>().len(),
        counts.len(),
        faces.len(),
    );
    assert_eq!(v + f, e + 2, "{label}: V{v} - E{e} + F{f} is not 2");
}

/// Vertex degrees, via the undirected edge set.
fn degrees(faces: &[Vec<usize>]) -> Vec<usize> {
    let mut degrees = vec![0; faces.iter().flatten().max().unwrap() + 1];
    for [a, b] in edge_counts(faces).into_keys() {
        degrees[a] += 1;
        degrees[b] += 1;
    }
    degrees
}

fn key_from_faces(faces: Faces, label: &str) -> String {
    // `degrees` sizes by the largest vertex id, so a numbering gap would pad the key with degree-0 vertices.
    // `assert_sphere` counts distinct vertices, so Euler balances across a gap and misses it.
    let mut faces = renumber(&faces);
    assert_sphere(&faces, label);
    let degrees = degrees(&faces);
    orient_faces(&mut faces);
    let signatures: Vec<FaceTypeSignature> = neighbor_type_signatures(&faces)
        .into_iter()
        .zip(&faces)
        .map(|(neighbor_sides, face)| FaceTypeSignature {
            side_count: face.len(),
            neighbor_sides,
        })
        .collect();
    crate::polydex::key(&degrees, &signatures)
}

/// Replaces one face with a pyramid over it, i.e. Conway `k` on a single face.
fn augment(faces: &[Vec<usize>], face: usize) -> Faces {
    let apex = faces.iter().flatten().max().unwrap() + 1;
    let mut out: Faces = faces.to_vec();
    let base = out.remove(face);
    for k in 0..base.len() {
        out.push(vec![base[k], base[(k + 1) % base.len()], apex]);
    }
    out
}

/// Joins two solids along an `n`-gon face, dropping the seam face from each.
/// The rotational offset is a free choice here because every seam lies on an antiprism, which has full cyclic symmetry.
fn glue(a: &[Vec<usize>], b: &[Vec<usize>], n: usize) -> Faces {
    let seam_a = a.iter().position(|f| f.len() == n).unwrap();
    let seam_b = b.iter().position(|f| f.len() == n).unwrap();
    let offset = a.iter().flatten().max().unwrap() + 1;
    // Reversed, so the two surfaces close up rather than fold onto each other.
    let seam: HashMap<usize, usize> = b[seam_b]
        .iter()
        .enumerate()
        .map(|(i, &v)| (v, a[seam_a][(n - i) % n]))
        .collect();
    let others = |faces: &[Vec<usize>], seam: usize| -> Faces {
        faces
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != seam)
            .map(|(_, f)| f.clone())
            .collect()
    };
    let mut all = others(a, seam_a);
    all.extend(others(b, seam_b).iter().map(|f| {
        f.iter()
            .map(|v| *seam.get(v).unwrap_or(&(v + offset)))
            .collect()
    }));
    renumber(&all)
}

/// The seven Johnson files netlib gets wrong, rebuilt from files it gets right.
fn repaired(index: usize, read: impl Fn(usize) -> Faces) -> Option<Faces> {
    Some(match index {
        // J22-J25 each lose one of their antiprism's two triangle rings.
        66 => glue(&read(47), &antiprism(6), 6),
        67 => glue(&read(48), &antiprism(8), 8),
        68 => glue(&read(49), &antiprism(10), 10),
        69 => glue(&read(50), &antiprism(10), 10),
        // J26 is a three-faces-to-an-edge mess with no clean parent, so spell it out.
        // Two triangular prisms square-glued a quarter turn apart: `0..=3` the discarded seam, `4,5` and `6,7` the ridges.
        70 => vec![
            vec![0, 1, 4],
            vec![2, 3, 5],
            vec![1, 4, 5, 2],
            vec![0, 3, 5, 4],
            vec![1, 2, 6],
            vec![3, 0, 7],
            vec![2, 3, 7, 6],
            vec![0, 1, 6, 7],
        ],
        // netlib's J37 is closed and Euler-consistent, but it is the orthobicupola, the rhombicuboctahedron.
        // The two have identical vertex figures, so only building the gyro version and splitting the face signatures catches it.
        81 => {
            // 0..8 bottom octagon, 8..16 top octagon, 16..20 and 20..24 the caps.
            let (bottom, top) = (0, 8);
            let mut out: Faces = (0..8)
                .map(|i| vec![bottom + i, bottom + (i + 1) % 8, top + (i + 1) % 8, top + i])
                .collect();
            for (ring, cap, turn) in [(bottom, 16, 0), (top, 20, 1)] {
                for j in 0..4 {
                    let edge = |k: usize| ring + (2 * j + k + turn) % 8;
                    out.push(vec![edge(0), edge(1), cap + j]);
                    out.push(vec![edge(1), edge(2), cap + (j + 1) % 4, cap + j]);
                }
                out.push((0..4).map(|j| cap + j).collect());
            }
            out
        }
        // J64's last three faces are the same triangle repeated, so rebuild it by augmenting J63.
        108 => {
            let j63 = read(107);
            let mut by_edge: HashMap<[usize; 2], Vec<usize>> = HashMap::new();
            for (f, face) in j63.iter().enumerate() {
                for k in 0..face.len() {
                    let (a, b) = (face[k], face[(k + 1) % face.len()]);
                    by_edge.entry([a.min(b), a.max(b)]).or_default().push(f);
                }
            }
            // The augmentation site is the one triangle bordered by three pentagons.
            let sites: Vec<usize> = (0..j63.len())
                .filter(|&f| {
                    j63[f].len() == 3
                        && (0..3).all(|k| {
                            let (a, b) = (j63[f][k], j63[f][(k + 1) % 3]);
                            by_edge[&[a.min(b), a.max(b)]]
                                .iter()
                                .all(|&g| g == f || j63[g].len() == 5)
                        })
                })
                .collect();
            assert_eq!(sites.len(), 1, "J63 augmentation site is not unique");
            augment(&j63, sites[0])
        }
        _ => return None,
    })
}

/// `n`-gonal prism: two `n`-gons joined by a band of squares.
fn prism(n: usize) -> Faces {
    let mut out: Faces = (0..n)
        .map(|i| vec![i, (i + 1) % n, n + (i + 1) % n, n + i])
        .collect();
    out.push((0..n).collect());
    out.push((n..2 * n).collect());
    out
}

/// `n`-gonal antiprism: two `n`-gons offset by half a step, joined by `2n` triangles.
fn antiprism(n: usize) -> Faces {
    let mut out: Faces = Vec::new();
    for i in 0..n {
        out.push(vec![i, (i + 1) % n, n + i]);
        out.push(vec![n + i, n + (i + 1) % n, (i + 1) % n]);
    }
    out.push((0..n).collect());
    out.push((n..2 * n).collect());
    out
}

/// Greek-prefixed polygon adjective, for naming the generated prism families.
fn polygon(n: usize) -> &'static str {
    [
        "Triangular",
        "Square",
        "Pentagonal",
        "Hexagonal",
        "Heptagonal",
        "Octagonal",
        "Enneagonal",
        "Decagonal",
        "Hendecagonal",
        "Dodecagonal",
    ][n - 3]
}

/// The thirteen Archimedean solids, built from the app's own Conway operators.
/// netlib ships a 2D net but no `:solid` block for any of them.
fn conway_built(index: usize) -> Option<Polyhedron> {
    let mut p = match index {
        9 => Polyhedron::preset(&Pyramid(3)),
        10 | 11 | 13 | 14 | 15 => Polyhedron::preset(&Prism(4)),
        12 => Polyhedron::preset(&Octahedron),
        16 | 17 | 19 | 20 | 21 => Polyhedron::preset(&Dodecahedron),
        18 => Polyhedron::preset(&Icosahedron),
        _ => return None,
    };
    match index {
        9 | 11 | 12 | 17 | 18 => {
            p.truncate(0);
        }
        10 | 16 => p.ambo_contract(),
        13 | 19 => p.expand(),
        // Bevel is `ta`: ambo first, then truncate.
        14 | 20 => {
            p.ambo_contract();
            p.truncate(0);
        }
        15 | 21 => p.snub(),
        _ => unreachable!(),
    }
    Some(p)
}

/// Turns a netlib name into `(Wikipedia title, Johnson number)`.
/// netlib's names are lowercase, parenthesize chirality and Johnson numbers, use older conventions, and hold two typos.
fn rename(netlib: &str) -> (String, Option<String>) {
    let johnson = netlib
        .split_once("(J")
        .and_then(|(_, rest)| rest.strip_suffix(')'))
        .map(|n| format!("J{n}"));
    let mut name = netlib
        .split(" (")
        .next()
        .unwrap()
        .replace("dipyramid", "bipyramid");
    name = match name.as_str() {
        "great rhombicuboctahedron" => "truncated cuboctahedron".into(),
        "great rhombicosidodecahedron" => "truncated icosidodecahedron".into(),
        "trapezoidal icositetrahedron" => "deltoidal icositetrahedron".into(),
        "trapezoidal hexecontahedron" => "deltoidal hexecontahedron".into(),
        "hexakis octahedron" => "disdyakis dodecahedron".into(),
        "hexakis icosahedron" => "disdyakis triacontahedron".into(),
        "elongated pentagonal rotunds" => "elongated pentagonal rotunda".into(),
        "pentagonal orthocupolarontunda" => "pentagonal orthocupolarotunda".into(),
        _ => name,
    };
    let mut chars = name.chars();
    let title = chars.next().unwrap().to_uppercase().collect::<String>() + chars.as_str();
    (title, johnson)
}

/// Whether en.wikipedia.org has an article at this title, checked once via the query API on 2026-10-05.
/// 139 of 141 resolved; the two largest antiprisms are only rows in the general antiprism article.
fn has_article(title: &str) -> bool {
    !matches!(title, "Hendecagonal antiprism" | "Dodecagonal antiprism")
}

fn category(index: usize) -> &'static str {
    match index {
        0..=4 => "Platonic",
        9..=21 => "Archimedean",
        32..=44 => "Catalan",
        _ => "Johnson",
    }
}

#[test]
#[ignore = "one-off table generator; prints rows for crate::polydex::TABLE"]
fn gen_polydex() {
    let source = source();
    let read = |index: usize| -> Faces {
        let (_, _, faces) = source
            .iter()
            .find(|(i, ..)| *i == index)
            .unwrap_or_else(|| panic!("no vendored face list for {index}"));
        faces.clone()
    };
    // (key, Wikipedia title, category); the title doubles as the display name.
    let mut rows: Vec<(String, String, String)> = Vec::new();

    // Prisms and antiprisms are infinite families, so generate them rather than read: netlib stops at ten sides.
    // The two that coincide with Platonic solids are skipped, so the cube and octahedron keep their Platonic names.
    for n in 3..=12 {
        for (faces, kind) in [(prism(n), "Prism"), (antiprism(n), "Antiprism")] {
            if (n == 4 && kind == "Prism") || (n == 3 && kind == "Antiprism") {
                continue;
            }
            let name = format!("{} {}", polygon(n), kind.to_lowercase());
            rows.push((key_from_faces(faces, &name), name, kind.to_string()));
        }
    }

    for (index, name, vendored) in source.iter().map(|(i, n, f)| (*i, n, f.clone())) {
        let label = format!("[{index}] {name}");

        let key = match (conway_built(index), repaired(index, read)) {
            (Some(p), _) => p.polydex_key(),
            (_, Some(faces)) => key_from_faces(faces, &label),
            // Entry 32 is a 2D net too, and the triakis tetrahedron is `kT`.
            _ if index == 32 => {
                let tetra = read(0);
                key_from_faces((0..4).fold(tetra, |f, _| augment(&f, 0)), &label)
            }
            _ => key_from_faces(vendored, &label),
        };

        let (title, johnson) = rename(name);
        let category = match johnson {
            Some(n) => format!("Johnson {n}"),
            None => category(index).to_string(),
        };
        rows.push((key, title, category));
    }

    let mut collisions = 0;
    for i in 0..rows.len() {
        for j in i + 1..rows.len() {
            if rows[i].0 == rows[j].0 {
                collisions += 1;
                println!("COLLISION {} <-> {}\n  {}", rows[i].1, rows[j].1, rows[i].0);
            }
        }
    }

    // Ready to paste into `crate::polydex::TABLE`.
    for (key, name, category) in &rows {
        let wiki = if has_article(name) { name } else { "" };
        println!(
            "ROW    Entry {{ key: {key:?}, name: {name:?}, category: {category:?}, wiki: {wiki:?} }},"
        );
    }
    assert_eq!(
        collisions,
        0,
        "key does not separate all {} solids",
        rows.len()
    );
}
