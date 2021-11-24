// Solver that tries to find a perfect fit of the pieces by treating the problem
// as a "rectangle packing" problem, using precomputed rectangles.
// Note that this only works for problems that require a perfect packing (4*totems == width*height).

// At a high level:
//   - (offline) precompute rectangles that can be made from different combinations of totems
//   - find possible sets of rectangles that have costs that sum up to our challenge's totem bag
//   - try to pack those rectangles in the given width x height grid.
//
// For this last step, we make use of a Monte-Carlo Tree Search, from this link:
// https://www.researchgate.net/publication/343895750_Monte_carlo_tree_search_on_perfect_rectangle_packing_problem_instances
// The basic idea is to consider, for each possible rectangle "move", the average or maximal
// score obtainable following that move based on a few simulations, and take the one with the highest
// score. Iterate until one of the simulations returns a perfect packing. The score in this
// case is the depth that we were able to reach in the simulation before getting stuck.
//
// A rectangle "move" here is deterministically made based on a common bin packing heuristic:
// Bottom-Left, where the lowest possible placement is chosen, left-aligned. Simulations just pick
// rectangles at random and also places them with this heuristic.

use crate::{
    game_interface::{TotemAnswer, TotemBag},
    max_rects,
    rect_inventory::{RectangleInventory, RectangleMetadata},
    solver::Solver,
    subset_sum,
};
use rand::{
    self,
    seq::SliceRandom,
    distributions::Distribution,
    Rng,
    SeedableRng,
};

#[derive(Clone)]
struct Placement {
    x: usize,
    y: usize,
    rect: RectangleMetadata,
}

#[derive(Clone)]
struct State {
    width: usize,
    height: usize,

    free_space: max_rects::MaxRects,
    placements: Vec<Placement>,
    // Index of rectangles that are left to place.
    indices_to_place: Vec<usize>,
}

struct SimulationResult {
    // How many rectangles we were able to place before getting stuck.
    depth: usize,
    solution: Option<Vec<Placement>>,
}

impl State {
    fn new(width: usize, height: usize, rectangles: &Vec<RectangleMetadata>) -> Self {
        State {
            width: width, height: height,
            free_space: max_rects::MaxRects::new(width, height),
            placements: Vec::new(),
            // All rectangles should be placed -- they sum up to our totem bag.
            indices_to_place: (0..rectangles.len()).collect(),
        }
    }

    // Place a rectangle based on the Bottom-Left heuristic, at its lowest possible position, left-aligned.
    fn bottom_left_place(&mut self, indices_to_place_idx: usize, rect: &RectangleMetadata) -> Option<()> {
        let (x, y) = self.free_space.bottom_left_insert(rect.dims.width, rect.dims.height)?;
        self.placements.push(Placement { x: x, y: y, rect: *rect });
        self.indices_to_place.swap_remove(indices_to_place_idx);
        Some(())
    }

    fn random_legal_move(&mut self, rng: &mut rand::rngs::SmallRng,
                         rectangles: &Vec<RectangleMetadata>) -> Option<()> {
        let idx_dist = rand::distributions::Uniform::from(0..self.indices_to_place.len());
        let i = idx_dist.sample(rng);
        let rect_index = self.indices_to_place[i];
        let mut rect = rectangles[rect_index].clone();
        if !rect.is_square() && rng.gen::<bool>() {  // Rotate the initial rectangle.
            rect.rotate();
        }
        if let Some(()) = self.bottom_left_place(i, &rect) {
            Some(())
        } else if !rect.is_square() {  // Try the other rotation, too.
            rect.rotate();
            self.bottom_left_place(i, &rect)  // If it fails again, we can't place this rectangle at all.
        } else {
            None
        }
    }

    // Place a random rectangle until we are stuck (or find a solution!)
    fn simulate(&mut self, rng: &mut rand::rngs::SmallRng,
                rectangles: &Vec<RectangleMetadata>) -> SimulationResult {
        let mut depth = 0;
        while !self.indices_to_place.is_empty() {
            if let Some(()) = self.random_legal_move(rng, rectangles) {
                depth += 1;
            } else {
                break;
            }
        }
        SimulationResult {
            depth: depth,
            solution: if self.placements.len() == rectangles.len() { Some(self.placements.clone()) } else { None }
        }
    }
}

// From our placed rectangles, recover the individual totems and their coordinates.
fn convert_solution(placements: &Vec<Placement>, inventory: &RectangleInventory) -> Vec<TotemAnswer> {
    let mut answer = Vec::new();
    for placement in placements {
        let mut rect = inventory.get_rectangle(&placement.rect).clone();
        if placement.rect.dims.width != rect.dims.width {  // rectangle is rotated
            rect = rect.rotate();
        }
        for totem in &rect.totems {
            answer.push(totem.offset_by(placement.x, placement.y));
        }
    }
    answer
}

// Monte carlo tree search for rectangle packing.
// Based on:
// From https://www.researchgate.net/publication/343895750_Monte_carlo_tree_search_on_perfect_rectangle_packing_problem_instances
fn mcts_packing(width: usize, height: usize, rectangles: &Vec<RectangleMetadata>, n_rolls: usize) -> Option<Vec<Placement>> {
    let mut state = State::new(width, height, rectangles);
    let mut rng = rand::rngs::SmallRng::from_entropy();
    loop {
        let mut best_state: Option<State> = None;
        let mut best_score = 0f32;
        for i in 0..state.indices_to_place.len() {
            let rect_index = state.indices_to_place[i];
            let mut rect = rectangles[rect_index];
            let rotations = if rect.is_square() { 1 } else { 2 };
            for _ in 0..rotations {
                let mut current_state = state.clone();
                if let Some(()) = current_state.bottom_left_place(i, &rect) {
                    let mut depths = Vec::new();
                    for _ in 0..n_rolls {
                        let mut state_sim = current_state.clone();
                        let result = state_sim.simulate(&mut rng, rectangles);
                        if let Some(solution) = result.solution {
                            return Some(solution);
                        }
                        depths.push(result.depth);
                    }

                    if !depths.is_empty() {
                        // max:
                        //let score = *depths.iter().max().unwrap() as f32;
                        // avg:
                        let total: usize = depths.iter().sum();
                        let score = total as f32 / depths.len() as f32;

                        if score > best_score {
                            best_score = score;
                            best_state = Some(current_state.clone());
                        }
                    }
                }
                rect.rotate();
            }
        }
        if let Some(best_state) = best_state {
            state = best_state;
        } else {
            return None;
        }
    }
}

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
        let mut rng = rand::rngs::SmallRng::from_entropy();
        let mut all_rectangles = self.inventory.available_rectangles(bag);
        let mut had_slns = false;
        for _ in 0..5 {  // Try a couple of times, shuffling rectangles can help with the subset sum.
            all_rectangles.shuffle(&mut rng);
            let it = subset_sum::MultiDimSubsetSumIterator::new(bag, &self.inventory,
                &all_rectangles, /*max_backtracks=*/50000);
            for rectangles_sln in it.take(5) {  // Check a few rectangle combinations
                had_slns = true;
                if let Some(sln) = mcts_packing(width, height, &rectangles_sln, /*n_rolls=*/7) {
                    return Some(convert_solution(&sln, &self.inventory));
                }
            }
        }
        if had_slns {
            println!("Had candidates, none could be packed.");
        }
    None
    }
}
