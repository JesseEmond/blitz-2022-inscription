// Solver that picks the next totem based on the one that maximizes the number of "touchpoints",
// i.e. other totem sides it touches (or borders). The intuition is that this is a very lightweight
// way to minimize the unpatchable holes that get created.
// When many shape rotations share the same number of "touchpoints", one is picked at random.
// Running multiple times can increase the chances of packing, so running each simulation fast is
// important.

// In practice, this greedy algorithm seems sufficient to solve optimally most levels that don't
// require an exact pack (although we did get a lucky 256 totems pack on the server, once!)

use crate::{
    game_interface::{Totem, TotemAnswer, TotemBag, TOTEMS},
    shape_info::ShapeVariant,
    solver::Solver,
};
use rand::{
    self,
    seq::SliceRandom,
};

struct Board {
    width: usize,
    height: usize,
    // For each row, a bitmask of each square that is in use.
    // (note: 0b1 would be to the very right, or x=63).
    masked_grid: Vec<u64>,
    // For each square, the amount of squares with totems that it has as neighbors.
    touchpoints: Vec<Vec<u32>>,
    // Totems placed so far.
    totems: Vec<TotemAnswer>,
    // For each x, the first y that has no totem on it yet.
    // Used to speed up finding a spot where a shape could fit.
    first_unset_y_at_x: Vec<usize>,
}

impl Board {
    fn new(width: usize, height: usize, answer_size: usize) -> Board {
        assert!(width <= 64);
        let mut touchpoints = vec![vec![0; width]; height];
        // Treat borders as touchpoints
        for x in 0..width {
            touchpoints[0][x] += 1;
            touchpoints[height - 1][x] += 1;
        }
        for y in 0..height {
            touchpoints[y][0] += 1;
            touchpoints[y][width - 1] += 1;
        }
        touchpoints[0][0] += 100; // Give (0,0) a big boost to ensure we set it.
        Board {
            width,
            height,
            // Because we always check 4 pre-made rows of masks for totems when checking for a fit (for speed),
            // need some padding.
            masked_grid: vec![0; height + 3],
            touchpoints,
            totems: Vec::with_capacity(answer_size),
            first_unset_y_at_x: vec![0; width],
        }
    }

    fn is_set(&self, x: usize, y: usize) -> bool {
        let mask = 1u64 << (63 - x);
        mask & self.masked_grid[y] != 0
    }

    fn mark(&mut self, shape: &ShapeVariant, left_x: usize, bottom_y: usize) {
        for (x, y) in &shape.coords {
            let x = *x + left_x;
            let y = *y + bottom_y;
            if y > 0 {
                self.touchpoints[y - 1][x] += 1;
            }
            if y + 1 < self.height {
                self.touchpoints[y + 1][x] += 1;
            }
            if x > 0 {
                self.touchpoints[y][x - 1] += 1;
            }
            if x + 1 < self.width {
                self.touchpoints[y][x + 1] += 1;
            }
            let mut unset_y = self.first_unset_y_at_x[x];
            if unset_y == y {
                while unset_y < self.height && self.is_set(x, unset_y) {
                    unset_y += 1;
                }
                self.first_unset_y_at_x[x] = unset_y;
            }
        }
        for dy in 0..shape.height {
            let y = bottom_y + dy as usize;
            let mask = shape.mask_at(left_x, dy as usize);
            self.masked_grid[y] |= mask;
        }
        let shape = shape.offset_by(left_x, bottom_y);
        self.totems.push(TotemAnswer::new(shape.shape, shape.coords));
    }

    fn fits(&self, shape: &ShapeVariant, left_x: usize, bottom_y: usize) -> bool {
        let mut fit = 0;
        for dy in 0..4 {  // constant loop size for speed, shapes are padded if needed.
            let y = bottom_y + dy as usize;
            let shape_mask = shape.mask_at(left_x, dy as usize);
            let board_mask = unsafe { self.masked_grid.get_unchecked(y) };  // Not great, but faster.
            fit |= shape_mask & board_mask;
        }
        return fit == 0;
    }

    fn num_touchpoints(&self, shape: &ShapeVariant, left_x: usize, bottom_y: usize) -> u32 {
        let mut total = 0;
        for (x, y) in &shape.coords {
            let x = *x + left_x;
            let y = *y + bottom_y;
            total += self.touchpoints[y][x];
        }
        total
    }

    // Minimum 'y' position where a shape could fit at a given 'x' position.
    // Trivially this would be 0, but by keeping track of free columns we can speed things up.
    fn min_y_for_shape(&self, shape: &ShapeVariant, left_x: usize) -> usize {
        let mut min_y = 0;
        for (dx, dy) in &shape.coords {
            let x = left_x + *dx;
            if self.first_unset_y_at_x[x] > min_y + *dy {
                min_y = self.first_unset_y_at_x[x] - *dy;
            }
        }
        min_y
    }

    // Find the first 'y' where a shape could fit at a given 'x', if possible.
    // Returns whether we could fit the shape.
    fn move_first_fit_above(&self, shape: &ShapeVariant, left_x: usize, out_y: &mut usize) -> bool {
        let min_y = self.min_y_for_shape(shape, left_x);
        for y in min_y..(self.height + 1 - shape.height as usize) {
            if self.fits(shape, left_x, y) {
                *out_y = y;
                return true;
            }
        }
        false
    }
}

struct Placement {
    totem: Totem,
    rotation_index: usize,
    x: usize,
    y: usize,
}

fn try_gravity_greedy_fit(board: &mut Board, mut bag: TotemBag) -> Option<Vec<TotemAnswer>> {
    let mut rng = rand::thread_rng();
    let mut options = Vec::with_capacity(7 * 4 * board.width);  // 7 shapes, 4 rotations max, 'width' x positions.
    loop {
        options.clear();
        let mut shapes_left = 0;
        let mut max_touchpoints = 0;

        for totem in TOTEMS {
            let n_totem = bag[totem];
            shapes_left += n_totem;
            if n_totem > 0 {
                for (rot_idx, variant) in ShapeVariant::get_rotations(&totem).iter().enumerate() {
                    for dx in 0..(board.width as i32 + 1 - variant.width as i32) {
                        let x = dx as usize;
                        let mut y = 0;
                        if board.move_first_fit_above(&variant, x, &mut y) {
                            let touchpoints = board.num_touchpoints(&variant, x, y);
                            if touchpoints > max_touchpoints {
                                max_touchpoints = touchpoints;
                                options.clear();  // new better options found, restart.
                            }
                            if touchpoints == max_touchpoints {
                                options.push(Placement {
                                    totem: variant.shape, rotation_index: rot_idx,
                                    x: x, y: y
                                    });
                            }
                        }
                    }
                }
            }
        }
        if shapes_left == 0 {
            return Some(board.totems.clone());
        }
        if options.is_empty() {
            return None;
        }
        let placement = options.choose(&mut rng).unwrap();
        let rotations = ShapeVariant::get_rotations(&placement.totem);
        let shape = rotations.iter().nth(placement.rotation_index).unwrap();
        board.mark(&shape, placement.x, placement.y);
        bag[shape.shape] -= 1;
    }
}

#[derive(Clone)]
pub struct GreedySolver {
}

impl Solver for GreedySolver {
    fn new() -> Self {
        Self { }
    }

    fn try_solve(&self, width: usize, height: usize, bag: &TotemBag) -> Option<Vec<TotemAnswer>> {
        let num_totems = bag.total();

        // Try multiple times due to the stochastic nature when multiple totems have the same number of
        // touchpoints. Doing so improves the packing %. Can only afford so many attempts at higher levels,
        // however.
        let attempts = if num_totems < 256 { 1000 } else { 100 };
        for _ in 0..attempts {
            if let Some(sln) = try_gravity_greedy_fit(&mut Board::new(width, height, num_totems), bag.clone()) {
                return Some(sln);
            }
        }
        None
    }
}
