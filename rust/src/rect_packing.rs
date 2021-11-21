// Based on https://www.researchgate.net/publication/343895750_Monte_carlo_tree_search_on_perfect_rectangle_packing_problem_instances

use crate::{
    game_interface::TotemAnswer,
    max_rects,
    rect_inventory::{RectangleInventory, RectangleMetadata},
    shape_info::ShapeDist,
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
    indices_to_place: Vec<usize>,
}

struct SimulationResult {
    depth: usize,
    solution: Option<Vec<Placement>>,
}

impl State {
    fn new(width: usize, height: usize, rectangles: &Vec<RectangleMetadata>) -> Self {
        State {
            width: width, height: height,
            free_space: max_rects::MaxRects::new(width, height),
            placements: Vec::new(),
            indices_to_place: (0..rectangles.len()).collect(),
        }
    }

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
fn do_mcts_packing(width: usize, height: usize, rectangles: &Vec<RectangleMetadata>, n_rolls: usize) -> Option<Vec<Placement>> {
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


// TODO doc do_mcts_packing + multi dim subset sum
// Solve multi-dimensional subset sum to find what rectangle combinations sum up to exactly our target 'dist'.
// Then, solve rectangle packing via MCTS for rectangle packing.
pub fn mcts_packing(width: usize, height: usize, dist: &ShapeDist, inventory: &RectangleInventory) -> Option<Vec<TotemAnswer>> {
    let mut rng = rand::rngs::SmallRng::from_entropy();
    let mut all_rectangles = inventory.available_rectangles(dist);
    let mut had_slns = false;
    for _ in 0..5 {
        all_rectangles.shuffle(&mut rng);
        let it = subset_sum::MultiDimSubsetSumIterator::new(dist, inventory, &all_rectangles, /*max_backtracks=*/50000);
        for rectangles_sln in it.take(5) {
            had_slns = true;
            if let Some(sln) = do_mcts_packing(width, height, &rectangles_sln, /*n_rolls=*/7) {
                return Some(convert_solution(&sln, inventory));
            }
        }
    }
    if had_slns {
        println!("Had candidates, none could be packed.");
    }
    None
}