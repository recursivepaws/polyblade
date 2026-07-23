use std::collections::{BTreeSet, HashMap, VecDeque};

/// Maps ever-growing color slots onto a bounded palette of display indices.
/// A live slot keeps its index while present, and a freed index recycles last so churned facetypes advance to fresh colors.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct PaletteAllocator {
    /// Palette index currently held by each live color slot.
    assigned: HashMap<usize, usize>,
    /// Available palette indices in hand-out order; the front is freshest and freed entries go to the back.
    free: VecDeque<usize>,
}

impl PaletteAllocator {
    /// Sets how many palette entries exist, keeping existing assignments and preference order.
    /// Returns whether anything changed; growth appends new indices, shrink drops out-of-range ones for reassignment.
    pub fn set_len(&mut self, len: usize) -> bool {
        let total = self.assigned.len() + self.free.len();
        if total == len {
            return false;
        }
        if len > total {
            self.free.extend(total..len);
        } else {
            self.free.retain(|&p| p < len);
            self.assigned.retain(|_, &mut p| p < len);
        }
        true
    }

    /// Reassigns palette indices for a new set of live slots.
    /// Survivors keep their index, vanished slots free theirs to the back, and newcomers take the front.
    pub fn reassign(&mut self, present: &BTreeSet<usize>) {
        let gone: Vec<usize> = self
            .assigned
            .keys()
            .copied()
            .filter(|s| !present.contains(s))
            .collect();
        for slot in gone {
            let palette = self.assigned.remove(&slot).unwrap();
            self.free.push_back(palette);
        }
        for &slot in present {
            if !self.assigned.contains_key(&slot) {
                // An exhausted palette hands out an out-of-range index that the render site wraps.
                // That knowingly reuses a color; the render site's debug_assert flags it in debug builds.
                let palette = match self.free.pop_front() {
                    Some(p) => p,
                    None => self.assigned.len(),
                };
                self.assigned.insert(slot, palette);
            }
        }
    }

    /// Palette index a slot maps to, falling back to 0 for an unassigned slot.
    pub fn palette_of(&self, slot: usize) -> usize {
        self.assigned.get(&slot).copied().unwrap_or(0)
    }
}
