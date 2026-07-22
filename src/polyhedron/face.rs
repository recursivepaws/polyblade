use super::palette::PaletteAllocator;
use crate::polyhedron::FaceId;
use std::collections::{BTreeSet, HashMap};

/// Per-face color bookkeeping, kept separate from `Render` since it tracks facetype identity, not physical simulation state.
/// Continuity is definitional, not matched: a face id keeps its color slot for as long as it lives.
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
    /// Persistent color slot per live face id; the single source of continuity across operations.
    slots: HashMap<FaceId, usize>,
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

    /// Assigns colors fresh, one slot per distinct signature, when there is no prior state to preserve.
    /// Wipes leftover id and palette state, since presets run operations internally before bootstrapping.
    pub fn bootstrap(&mut self, face_ids: &[FaceId], colors: Vec<usize>, next_color_slot: usize) {
        self.slots = face_ids
            .iter()
            .copied()
            .zip(colors.iter().copied())
            .collect();
        self.colors = colors;
        self.next_color_slot = next_color_slot;
        self.allocator = PaletteAllocator::default();
        // Seed a palette floor so the initial render is dense before any `set_palette_len`.
        self.allocator.set_len(next_color_slot);
        self.refresh_render_indices();
    }

    /// Carries colors across a structural change: surviving ids keep their slot, parented fresh ids inherit it, and remaining fresh ids get one new slot per signature.
    /// Normalization then enforces one color per signature, preferring inherited slots over ones minted this call, then most members, then smallest slot.
    pub fn finalize(
        &mut self,
        face_ids: &[FaceId],
        birth_parents: &HashMap<FaceId, FaceId>,
        signatures: &[FaceTypeSignature],
    ) {
        // Exact transfer by id, then by parent id.
        let mut colors: Vec<Option<usize>> = face_ids
            .iter()
            .map(|id| {
                self.slots.get(id).copied().or_else(|| {
                    birth_parents
                        .get(id)
                        .and_then(|parent| self.slots.get(parent))
                        .copied()
                })
            })
            .collect();

        // Mint for the genuinely new facetypes.
        let mut minted: Vec<usize> = Vec::new();
        let mut minted_by_signature: Vec<(&FaceTypeSignature, usize)> = Vec::new();
        for (i, color) in colors.iter_mut().enumerate() {
            if color.is_none() {
                let slot = match minted_by_signature
                    .iter()
                    .find(|(sig, _)| **sig == signatures[i])
                {
                    Some((_, slot)) => *slot,
                    None => {
                        let slot = self.next_color_slot;
                        self.next_color_slot += 1;
                        minted.push(slot);
                        minted_by_signature.push((&signatures[i], slot));
                        slot
                    }
                };
                *color = Some(slot);
            }
        }
        let mut colors: Vec<usize> = colors.into_iter().map(Option::unwrap).collect();

        // Normalization: one color per signature.
        let mut groups: Vec<(&FaceTypeSignature, Vec<usize>)> = Vec::new();
        for (i, sig) in signatures.iter().enumerate() {
            match groups.iter_mut().find(|(s, _)| *s == sig) {
                Some((_, members)) => members.push(i),
                None => groups.push((sig, vec![i])),
            }
        }
        for (_, members) in &groups {
            let mut votes: Vec<(usize, usize)> = Vec::new();
            for &i in members {
                match votes.iter_mut().find(|(slot, _)| *slot == colors[i]) {
                    Some((_, count)) => *count += 1,
                    None => votes.push((colors[i], 1)),
                }
            }
            if votes.len() > 1 {
                let winner = votes
                    .iter()
                    .min_by_key(|&&(slot, count)| {
                        (minted.contains(&slot), usize::MAX - count, slot)
                    })
                    .unwrap()
                    .0;
                for &i in members {
                    colors[i] = winner;
                }
            }
        }

        // Live ids adopt their final (possibly normalized) slots; dead ids fall away here.
        self.slots = face_ids
            .iter()
            .copied()
            .zip(colors.iter().copied())
            .collect();
        self.colors = colors;
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
