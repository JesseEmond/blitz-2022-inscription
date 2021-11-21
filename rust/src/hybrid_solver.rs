// Solve that picks the right solving strategy based on the level.
// It will also try multiple solution dimensions, in the order that
// would maximize the score.

use crate::{
    game_interface::{Answer, Question, TotemAnswer, TotemBag, TotemQuestion, TOTEMS},
    greedy_solver::GreedySolver,
    scoring::{score, Dims, OptimalDimensions},
    shape_info::ShapeVariant,
    solver::{macros::solver_boilerplate, Solver},
};
use std::cmp;

// Minimum dimensions needed to fit the individual totems in the bag.
// This is used to avoid trying e.g. a 2x2 board when we have an "L" piece, for instance.
// Note: this is done based on the smallest width rotation of each shape.
fn min_dimensions_needed(bag: &TotemBag) -> Dims {
    let mut dims = (0, 0);
    for totem in TOTEMS {
        if bag.contains(&totem) {
            let dim = ShapeVariant::minimum_dims(&totem);
            dims.0 = cmp::max(dims.0, dim.0);
            dims.1 = cmp::max(dims.1, dim.1);
        }
    }
    dims
}

pub struct HybridSolver {
    optimal_dims: OptimalDimensions,
    greedy: GreedySolver,
}

impl HybridSolver {
    /// Initialize your solver
    pub fn new() -> Self {
        Self {
            optimal_dims: OptimalDimensions::new(),
            greedy: GreedySolver::new(),
        }
    }

    /// Answer the question
    pub fn get_answer(&self, question: &Question) -> Answer {
        let num_totems = question.totems.len();
        println!("Received question with {} totems.", num_totems);

        let inferred_level = (num_totems as f64).log2().ceil() as usize;
        let (optimal_w, optimal_h) = self.optimal_dims.level_dims(inferred_level)[0];
        println!(
            "Optimal dims for level {} would be {}x{}, which would give score {}",
            inferred_level + 1,
            optimal_w,
            optimal_h,
            score(num_totems, optimal_w, optimal_h)
        );

        let bag = question.get_totem_bag();
        solver_boilerplate! {
            Answer::new(self.full_solve(&bag, inferred_level))
        }
    }

    // Try each optimal dimensions in order, trying to fit each one using 'try_solve'
    // to pick the right strategy.
    fn full_solve(&self, bag: &TotemBag, level: usize) -> Vec<TotemAnswer> {
        let min_dims = min_dimensions_needed(bag);
        let num_totems = bag.total();
        for (w, h) in self.optimal_dims.level_dims(level) {
            // Note: implicit assumption here that optimal_dims have the shortest dim first,
            // and that min_dimensions_needed also does so based on minimal width
            if min_dims.0 > *w || min_dims.1 > *h {
                println!("Skipping {}x{} (would have given {}), could not fit {}x{} totem",
                         *w, *h, score(num_totems, *w, *h), min_dims.0, min_dims.1);
                continue;
            }
            print!("Trying {}x{}... would give {}... ", *w, *h, score(num_totems, *w, *h));
            if let Some(fit) = self.try_solve(*w, *h, bag) {
                println!("OK!");
                return fit;
            } else if *w != *h {
                if let Some(fit) = self.try_solve(*h, *w, bag) {
                    // Because of our (0, 0) constraint, sometimes the rotation works.
                    // We typically run fast enough to just try both (non-squares optimal dims
                    // are mostly lower levels).
                    println!("OK!  (with rotation {}x{})", *h, *w);
                    return fit;
                }
            }
            println!("No fit found.");
        }
        println!("!!! FAILED TO FIND SOLUTION. Should increase ranges in 'optimal dims'.");
        // Fallback to greedy instead of returning nothing.
        let question = Question { totems: bag.expand().map(|t| TotemQuestion { shape: t }).collect() };
        GreedySolver::solve(&question).totems
    }
}

impl Solver for HybridSolver {
    fn solve(question: &Question) -> Answer {
        Self::new().get_answer(question)
    }

    fn try_solve(&self, width: usize, height: usize, bag: &TotemBag) -> Option<Vec<TotemAnswer>> {
        // TODO other implementations, too.
        self.greedy.try_solve(width, height, bag)
    }
}
