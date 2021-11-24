// Iterator to find a combination (with replacement) of rectangles that have costs
// that sum up _exactly_ to a given totem bag (our challenge). This is an easier task
// than trying to pack the rectangles as well, so we can treat this problem separately.

// Note this filename is a bit of a misnomer, we are solving a variant of multi dimensional
// subset sum, where instances can be used more than once. This is closer to a multi dimensional
// "coin change" problem, in a way.

use crate::{
    game_interface::{TotemBag, TOTEMS},
    rect_inventory::{RectangleInventory, RectangleMetadata},
};
use std::collections::HashSet;

type TotemBagHash = u64;

// Encode a distribution of 7 totems as a u64 (assuming MAX 512 qty per totem).
fn hash_bag(bag: &TotemBag) -> TotemBagHash {
    let mut h = 0u64;
    let mut mult = 1u64;
    for totem in TOTEMS {
        h += bag[totem] as u64 * mult;
        mult *= 513u64;  // We only care about up to 512 shapes, so can encode like this.
    }
    h
}

// Find combinations of 'rectangles' that exactly sum up to a cost of 'bag'.
// Note that subset sum is a bit of a misnomer, we are allowed to reuse rectangles.
pub struct MultiDimSubsetSumIterator<'a> {
    inventory: &'a RectangleInventory,
    // Note: introducing randomness in rectangles ordering can be helpful.
    rectangles: &'a Vec<&'a RectangleMetadata>,

    bag: TotemBag,
    // Indices of 'rectangles' chosen.
    chosen_indices: Vec<usize>,
    // Current index within 'rectangles' we're considering for the next value of chosen_indices.
    current_index: usize,
    // For each level of chosen_indices, whether it is a deadend (no return values) or not.
    is_deadend: Vec<bool>,
    // TotemBags that we know lead to deadends.
    deadends: HashSet<TotemBagHash>,
    // Tracker of how many times we backtracked, to be able to exit early on unlucky problems.
    backtrack_count: usize,
    max_backtracks: usize,
}

impl<'a> MultiDimSubsetSumIterator<'a> {
    pub fn new(bag: &TotemBag, inventory: &'a RectangleInventory,
               rectangles: &'a Vec<&'a RectangleMetadata>,
               max_backtracks: usize) -> Self {
        MultiDimSubsetSumIterator {
            inventory: inventory,
            rectangles: rectangles,
            bag: bag.clone(),
            chosen_indices: Vec::new(),
            is_deadend: Vec::new(),
            current_index: 0,
            deadends: HashSet::new(),
            backtrack_count: 0,
            max_backtracks: max_backtracks,
        }
    }

    // Undo the previous attempt and go to the next rectangle, if there's any (if there's not, return None).
    fn backtrack(&mut self) -> Option<()> {
        self.backtrack_count += 1;
        if self.backtrack_count >= self.max_backtracks {
            return None;
        }
        loop {
            if let Some(index) = self.chosen_indices.pop() {
                let deadend = self.is_deadend.pop().unwrap();
                if deadend {
                    self.deadends.insert(hash_bag(&self.bag));
                }
                let rect = self.rectangles[index];
                let cost = self.inventory.get_cost(rect);
                self.bag.add(cost);
                self.current_index = index + 1;
                if self.current_index < self.rectangles.len() {
                    return Some(());  // There are other rectangles to try.
                }
            } else {
                return None;  // No more rectangles to try!
            }
        }
    }

    // Pick the rectangle at 'current_index'.
    fn pick_current(&mut self) {
        self.chosen_indices.push(self.current_index);
        self.is_deadend.push(true);
        let rect = self.rectangles[self.current_index];
        let cost = self.inventory.get_cost(rect);
        self.bag.subtract(cost);
    }

    // When a valid match is found.
    fn on_match_found(&mut self) -> Vec<RectangleMetadata> {
        for is_deadend in &mut self.is_deadend {
            *is_deadend = false;
        }
        self.backtrack_count = 0;
        self.chosen_indices.iter().map(|idx| self.rectangles[*idx].clone()).collect()
    }
}

impl Iterator for MultiDimSubsetSumIterator<'_> {
    type Item = Vec<RectangleMetadata>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bag.is_empty() {
            // If we previously output something, undo the current rectangle to move back up.
            self.backtrack()?;
        }
        loop {
            let rect = self.rectangles[self.current_index];
            let cost = self.inventory.get_cost(rect);
            if self.bag.can_afford(cost) {
                self.pick_current();
                if self.bag.is_empty() {
                    // We have a match!
                    return Some(self.on_match_found());
                }
                if self.deadends.contains(&hash_bag(&self.bag)) {
                    self.backtrack()?;
                }
            } else {  // Can't afford this one, next rectangle.
                self.current_index += 1;
                if self.current_index == self.rectangles.len() {   // No more available rectangles, backtrack.
                    self.backtrack()?;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_rectangles() {
        //                  I, J, L, O, S, T, Z
        let bag = TotemBag([4, 6, 0, 2, 0, 0, 0]);
        let inventory = RectangleInventory::from_scratch(/*max_area=*/20);
        let rectangles = inventory.available_rectangles(&bag);
        let it = MultiDimSubsetSumIterator::new(&bag, &inventory, &rectangles, /*max_backtracks=*/10000000);
        let rect_sums: Vec<Vec<RectangleMetadata>> = it.collect();
        assert_eq!(rect_sums.len(), 174);
    }
}