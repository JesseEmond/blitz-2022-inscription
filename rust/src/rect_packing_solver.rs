// Solver that tries to find a perfect fit of the pieces by treating the problem
// as a "rectangle packing" problem, using precomputed rectangles.
// Note that this only works for problems that require a perfect packing (4*totems == width*height).

// At a high level:
//   - (offline) precompute rectangles that can be made from different combinations of totems
//   - find possible sets of rectangles that have costs that sum up to our challenge's totem bag
//   - try to pack those rectangles in the given width x height grid.

use crate::{
    game_interface::{TotemAnswer, TotemBag},
    rect_inventory::RectangleInventory,
    solver::Solver,
};

#[derive(Clone)]
pub struct RectPackingSolver {
    inventory: RectangleInventory,
}

impl Solver for RectPackingSolver {
    fn new() -> Self {
        Self {
            inventory: RectangleInventory::from_precomputed(
                &"../src/precomputed_area_32.rects".to_string()),
        }
    }

    fn try_solve(&self, width: usize, height: usize, bag: &TotemBag) -> Option<Vec<TotemAnswer>> {
        // TODO: implement

        None
    }
}
