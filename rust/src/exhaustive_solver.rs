// Solver that tries to exhaustively search for a fit.
// The running time grows exponentially with the number of totems,
// so this should only be used for <= 8 totems.
use crate::{
    game_interface::{Point, Totem, TotemAnswer, TotemBag, TOTEMS},
    shape_info::ShapeVariant,
    solver::Solver,
};
use std::cmp;

struct ShapeAssigment {
    shape: Totem,
    coords: [Point; 4],
    prev_max_x: usize,
    prev_max_y: usize,
}

// (Note that this has very similar code to the greedy solver, ideally the Board representation
// would be merged).
struct Board {
    width: usize,
    height: usize,

    max_x: usize,
    max_y: usize,
    grid: Vec<Vec<bool>>,
    assigments: Vec<ShapeAssigment>,
}

impl Board {
    fn new(width: usize, height: usize, num_totems: usize) -> Self {
        let grid = vec![vec![false; width]; height];
        Board {
            width: width, height: height, grid: grid,
            max_x: 0, max_y: 0,
            assigments: Vec::with_capacity(num_totems),
        }
    }

    // Note: assumes that the shape fits on the board. Does not check.
    fn fits(&self, shape: &ShapeVariant, left_x: usize, bottom_y: usize) -> bool {
        for (x, y) in &shape.coords {
            if self.grid[bottom_y + *y][left_x + *x] {
                return false;
            }
        }
        true
    }

    // Find first 'y' position where a shape would fit, for a given 'x'.
    // If the shape can't fit at that x, return false.
    fn move_first_fit_above(&self, left_x: usize, shape: &ShapeVariant, y: &mut usize) -> bool {
        // Note: could do similarly to the greedy min_y logic here, but it's fast enough for <= 8 totems
        // without.
        let min_shape_y = 0;
        for current_y in min_shape_y..(self.height as i32 + 1 - shape.height as i32) {
            *y = current_y as usize;
            if self.fits(shape, left_x, *y) {
                return true;
            }
        }
        return false
    }

    fn mark(&mut self, shape: &ShapeVariant, left_x: usize, bottom_y: usize) {
        let prev_max_x = self.max_x;
        let prev_max_y = self.max_y;
        for (x, y) in &shape.coords {
            let x = left_x + *x;
            let y = bottom_y + *y;
            self.grid[y][x] = true;
            self.max_x = cmp::max(self.max_x, x);
            self.max_y = cmp::max(self.max_y, y);
        }
        let shape = shape.offset_by(left_x, bottom_y);
        self.assigments.push(ShapeAssigment { shape: shape.shape, coords: shape.coords,
                                              prev_max_x: prev_max_x, prev_max_y: prev_max_y });
    }

    fn unmark(&mut self, shape: &ShapeVariant, left_x: usize, bottom_y: usize) {
        let assigment = self.assigments.pop().unwrap();
        self.max_x = assigment.prev_max_x;
        self.max_y = assigment.prev_max_y;
        for (x, y) in &shape.coords {
            let x = left_x + *x;
            let y = bottom_y + *y;
            self.grid[y][x] = false;
        }
    }

    fn assigments_to_answer(&self) -> Vec<TotemAnswer> {
        self.assigments.iter().map(|a| { TotemAnswer::new(a.shape, a.coords) }).collect()
    }
}


// Tries to place each shape rotation in each 'x' position, recursively.
// If a given placement failed to solve, 'board' and 'bag' will go back to their input values.
fn recursive_solve(board: &mut Board, bag: &mut TotemBag) -> Option<Vec<TotemAnswer>> {
    let mut shapes_left = 0;
    for totem in TOTEMS.iter() {
        let n_totems = bag[totem];
        shapes_left += n_totems;
        if n_totems == 0 { continue }
        for variant in ShapeVariant::get_rotations(&totem) {
            let mut upper_dx = board.width as i32 + 1 - variant.width as i32;
            if board.assigments.len() == 0 {
                // For first shape, force (0, 0) to be set.
                upper_dx = cmp::min(upper_dx, 1);
                if !variant.coords.iter().any(|(x, y)| *x == 0 && *y == 0) {
                    continue;
                }
            }
            for dx in 0..upper_dx {
                let mut dy = 0;
                if board.move_first_fit_above(dx as usize, &variant, &mut dy) {
                    board.mark(&variant, dx as usize, dy);
                    bag[totem] -= 1;
                    if let Some(sln) = recursive_solve(board, bag) {
                        return Some(sln);
                    } else {
                        board.unmark(&variant, dx as usize, dy);
                        bag[totem] += 1;
                    }
                }
            }
        }
    }
    if shapes_left == 0 {
        Some(board.assigments_to_answer())
    } else {
        None
    }
}

pub struct ExhaustiveSolver {
}

impl Solver for ExhaustiveSolver {
    fn new() -> Self {
        Self { }
    }

    fn try_solve(&self, width: usize, height: usize, bag: &TotemBag) -> Option<Vec<TotemAnswer>> {
        let num_totems = bag.total();
        let mut board = Board::new(width, height, num_totems);
        let mut bag = bag.clone();
        recursive_solve(&mut board, &mut bag)
    }
}