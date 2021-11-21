use crate::{
    rect_inventory::{self, RectangleInventory, RectangleMetadata},
    shape_info::ShapeDist,
};
use std::collections::HashSet;

type EncodedShapeDist = u64;

// Encode a distribution of 7 totems as a u64 (assuming MAX 512 per totem).
fn encode(cost: &ShapeDist) -> EncodedShapeDist {
    let mut encoded = 0u64;
    let mut mult = 1u64;
    for i in 0..7 {
        encoded += cost[i] as u64 * mult;
        mult *= 513u64;  // We only care about up to 512 shapes, so can encode like this.
    }
    encoded
}

struct Bag {
    pieces: ShapeDist,
    code: EncodedShapeDist,
}

impl Bag {
    fn new(bag: &ShapeDist) -> Self {
        Bag {
            pieces: *bag,
            code: encode(bag),
        }
    }

    fn add(&mut self, cost: &ShapeDist) {
        for i in 0..7 {
            self.pieces[i] += cost[i];
        }
        self.code = encode(&self.pieces);
    }

    fn remove(&mut self, cost: &ShapeDist) {
        for i in 0..7 {
            self.pieces[i] -= cost[i];
        }
        self.code = encode(&self.pieces);
    }

    fn can_afford(&self, cost: &ShapeDist) -> bool {
        rect_inventory::can_afford(cost, &self.pieces)
    }

    fn is_empty(&self) -> bool {
        self.code == 0
    }
}

// Find combinations of 'rectangles' that exactly sum up to a cost of 'bag'.
pub struct MultiDimSubsetSumIterator<'a> {
    inventory: &'a RectangleInventory,
    rectangles: &'a Vec<&'a RectangleMetadata>,

    bag: Bag,
    // Indices of 'rectangles' chosen.
    chosen_indices: Vec<usize>,
    // Current index within 'rectangles' we're considering for the next value of chosen_indices.
    current_index: usize,
    // For each level of chosen_indices, whether it is a deadend (no return values) or not.
    is_deadend: Vec<bool>,
    // Shape distributions that we know lead to deadends.
    deadends: HashSet<EncodedShapeDist>,
    backtrack_count: usize,
    max_backtracks: usize,
}

impl<'a> MultiDimSubsetSumIterator<'a> {
    pub fn new(bag: &ShapeDist, inventory: &'a RectangleInventory, rectangles: &'a Vec<&'a RectangleMetadata>,
               max_backtracks: usize) -> Self {
        MultiDimSubsetSumIterator {
            inventory: inventory,
            // TODO: sort rectangles to have bigger ones (squares ideally) first?
            rectangles: rectangles,
            bag: Bag::new(bag),
            chosen_indices: Vec::new(),
            is_deadend: Vec::new(),
            current_index: 0,
            deadends: HashSet::new(),
            backtrack_count: 0,
            max_backtracks: max_backtracks,
        }
    }

    // Undo the previous attempt and go to the next rectangle, if there's any (if there's not, return false).
    fn backtrack(&mut self) -> Option<()> {
        self.backtrack_count += 1;
        if self.backtrack_count >= self.max_backtracks {
            return None;
        }
        loop {
            if let Some(index) = self.chosen_indices.pop() {
                let deadend = self.is_deadend.pop().unwrap();
                if deadend {
                    self.deadends.insert(self.bag.code);
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
        self.bag.remove(cost);
    }

    fn match_found(&mut self) -> Vec<RectangleMetadata> {
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
                    return Some(self.match_found());
                }
                if self.deadends.contains(&self.bag.code) {
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
        //         I, J, L, O, S, T, Z
        let bag = [0, 0, 0, 0, 0, 8, 8];
        let inventory = RectangleInventory::from_precomputed();
        let rectangles = inventory.available_rectangles(&bag);
        let it = MultiDimSubsetSumIterator::new(&bag, &inventory, &rectangles, 10000000);
        for i in it {
            println!("{} rectangles.", i.len());
        }
    }
}