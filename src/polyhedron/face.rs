use std::collections::{BTreeSet, HashMap, HashSet};

#[derive(Debug, Default, Clone, PartialEq)]
struct FaceCache {
    ancestors: Vec<HashSet<u64>>,
    colors: Vec<usize>,
    /// Side count of each snapshotted face, so `reconcile` can prefer same-facetype matches.
    side_counts: Vec<usize>,
}

/// Per-face color bookkeeping, kept separate from `Render` since it tracks facetype identity, not physical simulation state.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct FaceColoring {
    /// Palette-relative color slot per current face, parallel to `shape.cycles`.
    pub colors: Vec<usize>,
    /// Next unused color slot; monotonically increasing so new facetypes get distinct colors.
    next_color_slot: usize,
    /// Palette index per face, derived from `colors`; kept in sync wherever `colors` is set.
    pub render_indices: Vec<usize>,
    /// Palette index assigned to each currently-present color slot. A surviving slot keeps its
    /// entry, so a facetype's rendered color never changes while it stays on screen.
    palette_of_slot: HashMap<usize, usize>,
    /// Palette indices in allocation-preference order (front = used first), and implicitly the
    /// palette length. When a facetype disappears its entry moves to the back, so new facetypes
    /// advance to fresh colors instead of recycling a just-freed one.
    palette_order: Vec<usize>,
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
    /// Tells the coloring how many palette entries exist.
    /// Preserves the current preference order for still-valid entries and appends any newly-available ones at the end.
    pub fn set_palette_len(&mut self, len: usize) {
        if self.palette_order.len() == len {
            return;
        }
        let mut order: Vec<usize> = self
            .palette_order
            .iter()
            .copied()
            .filter(|&p| p < len)
            .collect();
        for p in 0..len {
            if !order.contains(&p) {
                order.push(p);
            }
        }
        self.palette_order = order;
    }

    /// Snapshots the current colors against a fresh ancestor set, as the baseline for the next `reconcile`.
    pub fn snapshot(&mut self, ancestors: Vec<HashSet<u64>>, side_counts: Vec<usize>) {
        self.cache = FaceCache {
            ancestors,
            colors: self.colors.clone(),
            side_counts,
        };
    }

    /// Assigns colors fresh, one slot per distinct signature; used when there's no prior state to preserve continuity from.
    pub fn bootstrap(&mut self, colors: Vec<usize>, next_color_slot: usize) {
        self.colors = colors;
        self.next_color_slot = next_color_slot;
        self.assign_render_indices();
    }

    /// Matches faces to a pre-mutation ancestry snapshot by Jaccard similarity.
    /// Results are then majority-voted per `FaceTypeSignature` to guarantee one color per facetype.
    ///
    /// `ancestors` is the post-mutation ancestry, one entry per current face.
    /// The pre-mutation baseline it's matched against is whatever `snapshot` last recorded.
    pub fn reconcile(&mut self, ancestors: Vec<HashSet<u64>>, signatures: &[FaceTypeSignature]) {
        let old = &self.cache;

        // (new_face, old_face, intersection, union, same_side) per candidate pair with any overlap.
        let mut candidates: Vec<(usize, usize, usize, usize, bool)> = Vec::new();
        for (i, a) in ancestors.iter().enumerate() {
            for (j, o) in old.ancestors.iter().enumerate() {
                let intersection = o.intersection(a).count();
                if intersection > 0 {
                    let union = o.union(a).count();
                    let same_side = old
                        .side_counts
                        .get(j)
                        .is_some_and(|&s| s == signatures[i].side_count);
                    candidates.push((i, j, intersection, union, same_side));
                }
            }
        }

        // Prefer a same-facetype ancestor first, then Jaccard similarity, then raw overlap count.
        // Same-side ranking keeps a surviving face (e.g. a triangle whose ancestry got flooded by
        // contracted neighbors) matched to its own facetype instead of a larger, better-overlapping one.
        candidates.sort_by(|&(_, _, ia, ua, sa), &(_, _, ib, ub, sb)| {
            let jaccard_a = ia as f64 / ua as f64;
            let jaccard_b = ib as f64 / ub as f64;
            sb.cmp(&sa)
                .then_with(|| jaccard_b.total_cmp(&jaccard_a))
                .then(ib.cmp(&ia))
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
            // Prefer the most common real (matched) color;
            // only mint a new slot if no face in this group matched anything.
            // Otherwise a facetype with more new faces than old ones
            // (e.g. expand's 8 triangles onto 4) could tie against `None` and lose its color.
            let winner = votes
                .iter()
                .filter(|(v, _)| v.is_some())
                .max_by_key(|(_, count)| *count)
                .map(|(v, _)| *v)
                .unwrap_or(None);

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
        self.assign_render_indices();
    }

    /// Maps each face's color slot to a palette index.
    /// A slot that is still present keeps its entry (a facetype never changes color while on screen).
    /// Each palette entry freed by a disappearing facetype moves to the back of `palette_order`,
    /// so newly-present slots draw the freshest colors first and only recycle a freed one once the rest are exhausted.
    fn assign_render_indices(&mut self) {
        let present: BTreeSet<usize> = self.colors.iter().copied().collect();

        // Send every palette entry freed this round to the back of the preference order.
        let mut freed: Vec<usize> = self
            .palette_of_slot
            .iter()
            .filter(|(slot, _)| !present.contains(slot))
            .map(|(_, &palette)| palette)
            .collect();
        freed.sort_unstable();

        for palette in freed {
            self.palette_order.retain(|&p| p != palette);
            self.palette_order.push(palette);
        }

        // Make sure there is always at least one entry per present facetype to hand out.
        for extra in self.palette_order.len()..present.len() {
            self.palette_order.push(extra);
        }

        let mut new_map: HashMap<usize, usize> = HashMap::new();
        let mut used: BTreeSet<usize> = BTreeSet::new();

        // Survivors keep their palette entry.
        for &slot in &present {
            if let Some(&palette) = self.palette_of_slot.get(&slot) {
                new_map.insert(slot, palette);
                used.insert(palette);
            }
        }

        // New facetypes take the first not-in-use entry in preference order.
        for &slot in &present {
            if let std::collections::hash_map::Entry::Vacant(entry) = new_map.entry(slot) {
                let palette = self
                    .palette_order
                    .iter()
                    .copied()
                    .find(|p| !used.contains(p))
                    .unwrap_or(0);
                used.insert(palette);
                entry.insert(palette);
            }
        }

        self.render_indices = self.colors.iter().map(|slot| new_map[slot]).collect();
        self.palette_of_slot = new_map;
    }
}
