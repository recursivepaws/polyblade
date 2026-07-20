use std::collections::HashSet;

#[derive(Debug, Default, Clone, PartialEq)]
struct FaceCache {
    ancestors: Vec<HashSet<u64>>,
    colors: Vec<usize>,
}

/// Per-face color bookkeeping, kept separate from `Render` since it tracks facetype identity, not physical simulation state.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct FaceColoring {
    /// Palette-relative color slot per current face, parallel to `shape.cycles`.
    pub colors: Vec<usize>,
    /// Next unused color slot; monotonically increasing so new facetypes get distinct colors.
    next_color_slot: usize,
    /// Dense render index per face, derived from `colors`; kept in sync wherever `colors` is set.
    pub render_indices: Vec<usize>,
    /// Pre-mutation snapshot of ancestors/colors, used to reconcile colors across a structural change.
    cache: FaceCache,
}

/// A face's "type": side count plus its neighbors' sorted side-count multiset.
/// Stable across structural changes, unlike a raw face index.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FaceTypeSignature {
    pub side_count: usize,
    pub neighbor_sides: Vec<usize>,
}

/// One distinct face "type" available to project through in Schlegel mode.
#[derive(Debug, Clone, PartialEq)]
pub struct FaceTypeOption {
    pub face_index: usize,
    pub signature: FaceTypeSignature,
    pub count: usize,
    pub label: String,
}

impl FaceColoring {
    /// Snapshots the current colors against a fresh ancestor set, as the baseline for the next `reconcile`.
    pub fn snapshot(&mut self, ancestors: Vec<HashSet<u64>>) {
        self.cache = FaceCache {
            ancestors,
            colors: self.colors.clone(),
        };
    }

    /// Assigns colors fresh, one slot per distinct signature; used when there's no prior state to preserve continuity from.
    pub fn bootstrap(&mut self, colors: Vec<usize>, next_color_slot: usize) {
        self.colors = colors;
        self.next_color_slot = next_color_slot;
        self.render_indices = dense_color_indices(&self.colors);
    }

    /// Matches faces to a pre-mutation ancestry snapshot by Jaccard similarity.
    /// Results are then majority-voted per `FaceTypeSignature` to guarantee one color per facetype.
    ///
    /// `ancestors` is the post-mutation ancestry, one entry per current face.
    /// The pre-mutation baseline it's matched against is whatever `snapshot` last recorded.
    pub fn reconcile(&mut self, ancestors: Vec<HashSet<u64>>, signatures: &[FaceTypeSignature]) {
        let old = &self.cache;

        // (new_face, old_face, intersection, union) per candidate pair with any overlap.
        let mut candidates: Vec<(usize, usize, usize, usize)> = Vec::new();
        for (i, a) in ancestors.iter().enumerate() {
            for (j, o) in old.ancestors.iter().enumerate() {
                let intersection = o.intersection(a).count();
                if intersection > 0 {
                    let union = o.union(a).count();
                    candidates.push((i, j, intersection, union));
                }
            }
        }
        // Rank by Jaccard similarity (descending), breaking ties by raw overlap count.
        candidates.sort_by(|&(_, _, ia, ua), &(_, _, ib, ub)| {
            let jaccard_a = ia as f64 / ua as f64;
            let jaccard_b = ib as f64 / ub as f64;
            jaccard_b.total_cmp(&jaccard_a).then(ib.cmp(&ia))
        });

        let mut matched_color: Vec<Option<usize>> = vec![None; ancestors.len()];
        let mut old_claimed = vec![false; old.ancestors.len()];
        for (i, j, ..) in candidates {
            if matched_color[i].is_none() && !old_claimed[j] {
                matched_color[i] = Some(old.colors[j]);
                old_claimed[j] = true;
            }
        }

        // Group by facetype and majority-vote one color per group.
        let mut groups: Vec<(FaceTypeSignature, Vec<usize>)> = Vec::new();
        for (i, sig) in signatures.iter().enumerate() {
            match groups.iter_mut().find(|(s, _)| s == sig) {
                Some((_, members)) => members.push(i),
                None => groups.push((sig.clone(), vec![i])),
            }
        }

        let mut new_colors = vec![0; ancestors.len()];
        for (_, members) in &groups {
            let mut votes: Vec<(Option<usize>, usize)> = Vec::new();
            for &i in members {
                match votes.iter_mut().find(|(v, _)| *v == matched_color[i]) {
                    Some((_, count)) => *count += 1,
                    None => votes.push((matched_color[i], 1)),
                }
            }
            let winner = votes.iter().max_by_key(|(_, count)| *count).unwrap().0;

            let color = winner.unwrap_or_else(|| {
                let slot = self.next_color_slot;
                self.next_color_slot += 1;
                slot
            });
            for &i in members {
                new_colors[i] = color;
            }
        }

        self.colors = new_colors;
        self.render_indices = dense_color_indices(&self.colors);
    }
}

/// Maps `colors`'s ever-growing values to a dense render index, so two facetypes never collide merely by being congruent mod `colors.len()`.
fn dense_color_indices(colors: &[usize]) -> Vec<usize> {
    let mut distinct = colors.to_vec();
    distinct.sort_unstable();
    distinct.dedup();
    colors
        .iter()
        .map(|slot| distinct.binary_search(slot).unwrap())
        .collect()
}
