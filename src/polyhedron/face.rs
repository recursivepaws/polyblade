use super::palette::PaletteAllocator;
use std::collections::{BTreeSet, HashSet};

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
    /// Color slot per current face, parallel to `shape.cycles`.
    pub colors: Vec<usize>,
    /// Next unused color slot; monotonically increasing so new facetypes get distinct colors.
    next_color_slot: usize,
    /// Cached palette index per face; derived from `colors` via `allocator`, read every frame by the renderer.
    pub render_indices: Vec<usize>,
    /// Maps color slots to stable, recyclable palette indices.
    allocator: PaletteAllocator,
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
    /// Tells the coloring how many palette entries exist, refreshing render indices if it changed.
    pub fn set_palette_len(&mut self, len: usize) {
        if self.allocator.set_len(len) {
            self.refresh_render_indices();
        }
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
        // Seed a palette floor so the initial render is dense before any `set_palette_len`.
        self.allocator.set_len(next_color_slot);
        self.refresh_render_indices();
    }

    /// Matches faces to a pre-mutation ancestry snapshot by Jaccard similarity.
    /// Results are then majority-voted per `FaceTypeSignature` to guarantee one color per facetype.
    ///
    /// `ancestors` is the post-mutation ancestry, one entry per current face.
    /// The pre-mutation baseline it's matched against is whatever `snapshot` last recorded.
    pub fn reconcile(&mut self, ancestors: Vec<HashSet<u64>>, signatures: &[FaceTypeSignature]) {
        let old = &self.cache;

        // (new_face, old_face, coverage, jaccard, same_side) per candidate pair with any overlap.
        // `coverage` is the fraction of the old face's ancestry inherited by the new face.
        let mut candidates: Vec<(usize, usize, f64, f64, bool)> = Vec::new();
        for (i, a) in ancestors.iter().enumerate() {
            for (j, o) in old.ancestors.iter().enumerate() {
                let intersection = o.intersection(a).count();
                if intersection > 0 {
                    let coverage = intersection as f64 / o.len() as f64;
                    let jaccard = intersection as f64 / o.union(a).count() as f64;
                    let same_side = old
                        .side_counts
                        .get(j)
                        .is_some_and(|&s| s == signatures[i].side_count);
                    candidates.push((i, j, coverage, jaccard, same_side));
                }
            }
        }

        // Rank by how fully the new face inherits the old face's ancestry, then same-side, then Jaccard.
        // Coverage keeps a face's color when its side count changes (truncation's square -> octagon), where side-count matching would misassign it.
        candidates.sort_by(|&(_, _, ca, ja, sa), &(_, _, cb, jb, sb)| {
            cb.total_cmp(&ca)
                .then_with(|| sb.cmp(&sa))
                .then_with(|| jb.total_cmp(&ja))
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
            // Prefer the most common matched color; mint a new slot only if nothing matched.
            // Otherwise a facetype with more new faces than old (e.g. expand's 8 triangles onto 4) ties against `None` and loses its color.
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
        self.refresh_render_indices();
    }

    /// Reassigns palette indices for the current slots and rebuilds the cached render indices.
    fn refresh_render_indices(&mut self) {
        let present: BTreeSet<usize> = self.colors.iter().copied().collect();
        self.allocator.reassign(&present);
        self.render_indices = self
            .colors
            .iter()
            .map(|&slot| self.allocator.palette_of(slot))
            .collect();
    }
}
