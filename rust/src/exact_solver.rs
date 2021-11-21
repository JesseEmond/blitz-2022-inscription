// Solver that tries to exhaustively search for a perfect fit.
use crate::{
    game_interface::{CoordinatePair, Totem, TotemAnswer},
    shape_info::{ShapeDist, ShapeVariant},
};
use std;
use std::cmp;

struct ShapeAssigment {
    shape: Totem,
    coords: [CoordinatePair; 4],
    prev_max_x: usize,
    prev_max_y: usize,
}

struct Board {
    width: usize,
    height: usize,
    // TODO: before placing a piece, check if it would bring the number of holes above the max possible


    max_x: usize,
    max_y: usize,
    grid: Vec<Vec<bool>>,
    first_unset_at_x: Vec<usize>,
    assigments: Vec<ShapeAssigment>,
}

impl Board {
    fn new(width: usize, height: usize, num_totems: usize) -> Self {
        let grid = vec![vec![false; width]; height];
        Board {
            width: width, height: height, grid: grid,
            max_x: 0, max_y: 0,
            assigments: Vec::with_capacity(num_totems),
            first_unset_at_x: vec![0; width],
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

    fn move_first_fit_above(&self, left_x: usize, shape: &ShapeVariant, y: &mut usize) -> bool {
        let mut min_shape_y: usize = 0;  // Min y position where the bottom of 'shape' could fit.
        for (shape_x, shape_y) in &shape.coords {
            let x = left_x + *shape_x;
            let mut first_unset_y = self.first_unset_at_x[x];
            if first_unset_y + *shape_y >= self.height {
                return false;  // No vertical space for this shape.
            }
            if first_unset_y > *shape_y {  // This is only informative if it's above our shape's init position
                // (also to avoid underflowing usizes)
                first_unset_y -= *shape_y;  // consider first_unset_y as the bottom of the shape
            }
            min_shape_y = cmp::max(min_shape_y, first_unset_y);
        }
        min_shape_y = 0;  // TODO this improves perfect packs -- there's a bug in the logic then
        for current_y in min_shape_y..(self.height + 1 - shape.height as usize) {
            *y = current_y;
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
            let mut unset_y = self.first_unset_at_x[x];
            if unset_y == y {
                while unset_y < self.height && self.grid[unset_y][x] {
                    unset_y += 1;
                }
                self.first_unset_at_x[x] = unset_y;
            }
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
            let mut unset_y = self.first_unset_at_x[x];
            if unset_y == y + 1 {
                while unset_y > 0 && !self.grid[unset_y-1][x] {
                    unset_y -= 1;
                }
                self.first_unset_at_x[x] = unset_y;
            }
        }
    }

    fn assigments_to_answer(&self) -> Vec<TotemAnswer> {
        self.assigments.iter().map(|a| { TotemAnswer::new(a.shape, a.coords.to_vec()) }).collect()
    }

    #[allow(dead_code)]
    fn debug_print(&self) {
        for y in (0..self.height).rev() {
            for x in 0..self.width {
                print!("{}", if self.grid[y][x] { '#' } else { '.' });
            }
            println!();
        }
        println!("{}x{}, {} totems", self.width, self.height, self.assigments.len());
        println!("first unset ys: {:?}", self.first_unset_at_x);
    }
}


// Very ugly code only used when "speedlogging" feature is set. Counts overall speed
// in our recursion function for how many times it looks at a totem.
#[cfg(feature = "speedlogging")]
static mut SHAPES_COUNTER: u64 = 0;
#[cfg(feature = "speedlogging")]
static mut LAST_TIME: Option<std::time::Instant> = None;
#[cfg(feature = "speedlogging")]
unsafe fn count_shape_seen() {
    SHAPES_COUNTER += 1;
    let time = LAST_TIME.unwrap().elapsed().as_secs_f64();
    if time >= 1.0 {
        let speed = SHAPES_COUNTER as f64 / time;
        println!("  [{} shapes/s]", speed);
        LAST_TIME = Some(std::time::Instant::now());
        SHAPES_COUNTER = 0;
    }
}


// If failed to solve, 'board' and 'dist' will go back to their input values.
// Used recursively.
fn try_solve(board: &mut Board, dist: &mut ShapeDist) -> Option<Vec<TotemAnswer>> {
    let mut shapes_left = 0;
    for totem in Totem::iter() {
        let n_totems = dist[*totem as usize];
        shapes_left += n_totems;
        if n_totems == 0 { continue }
        for variant in ShapeVariant::get_rotations(totem) {
            let mut upper_dx = board.width as i32 + 1 - variant.width as i32;
            if board.assigments.len() == 0 {
                // For first shape, force (0, 0) to be set.
                upper_dx = cmp::min(upper_dx, 1);
                if !variant.coords.iter().any(|(x, y)| *x == 0 && *y == 0) {
                    continue;
                }
            }
            for dx in 0..upper_dx {
                #[cfg(feature = "speedlogging")]
                unsafe { count_shape_seen() }

                let mut dy = 0;
                if board.move_first_fit_above(dx as usize, &variant, &mut dy) {
                    board.mark(&variant, dx as usize, dy);
                    dist[*totem as usize] -= 1;
                    if let Some(sln) = try_solve(board, dist) {
                        return Some(sln);
                    } else {
                        board.unmark(&variant, dx as usize, dy);
                        dist[*totem as usize] += 1;
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


pub fn solve(width: usize, height: usize, dist: ShapeDist) -> Option<Vec<TotemAnswer>> {
    #[cfg(feature = "speedlogging")]
    unsafe { LAST_TIME = Some(std::time::Instant::now()); }

    assert!(width < 32 && height < 32);  // TODO: change to 64?
    let num_totems: usize = dist.iter().sum();
    assert!(num_totems <= 64);  // TODO: increase?
    let mut board = Board::new(width, height, num_totems);
    try_solve(&mut board, &mut dist.clone())
}