// Solver that picks the right solving strategy based on the level.
// It will also try multiple solution dimensions, in the order that
// would maximize the score.

use crate::{
    exhaustive_solver::ExhaustiveSolver,
    game_interface::{Answer, Question, TotemAnswer, TotemBag, TotemQuestion, TOTEMS},
    greedy_solver::GreedySolver,
    scoring::{score, Dims, OptimalDimensions},
    shape_info::ShapeVariant,
    solver::{macros::solver_boilerplate, Solver},
    rect_packing_solver::RectPackingSolver,
};
use std::{cmp, thread};

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
    // Usually want this on, but can be useful to turn off when profiling.
    use_multithreading: bool,
    // Probably want this on on the server, but not when evaluating offline in a loop.
    verbose: bool,
    optimal_dims: OptimalDimensions,

    greedy: GreedySolver,
    exhaustive: ExhaustiveSolver,
    rect_packing: RectPackingSolver,
}

impl HybridSolver {
    pub fn with_options(multithreading: bool, verbose: bool) -> Self {
        Self {
            optimal_dims: OptimalDimensions::new(),
            use_multithreading: multithreading,
            verbose: verbose,
            greedy: GreedySolver::new(),
            exhaustive: ExhaustiveSolver::new(),
            rect_packing: RectPackingSolver::new(),
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
        self.greedy.solve(&question).totems
    }
}

macro_rules! multithread_solver {
    ( $x: expr, $w: ident, $h: ident, $bag: ident ) => {
        {
            // From tests, we think we're on a c5a.2xlarge, so 4 cores, 8 hyperthreaded.
            // As IIUC going up to 8 would hurt, since we're doing purely CPU processing
            // and not much IO:
            // https://www.credera.com/insights/whats-in-a-vcpu-state-of-amazon-ec2-in-2018
            let cores = 4-1;  // leave some breathing room with -1
            let mut handles = vec![];
            for _ in 0..cores {
                let bag = $bag.clone();
                let solver = $x.clone();
                handles.push(thread::spawn(move || {
                    solver.try_solve($w, $h, &bag)
                }));
            }
            for handle in handles {
                if let Some(sln) = handle.join().unwrap() {
                    return Some(sln);
                }
            }
            None
        }
    };
}

impl Solver for HybridSolver {
    fn new() -> Self {
        Self::with_options(/*multithreading=*/true, /*verbose=*/true)
    }

    fn solve(&self, question: &Question) -> Answer {
        self.get_answer(question)
    }

    fn try_solve(&self, width: usize, height: usize, bag: &TotemBag) -> Option<Vec<TotemAnswer>> {
        let num_totems = bag.total();

        // For <= 8, we can do an exhaustive search.
        let greedy = num_totems > 8;

        // TODO check if hard level & perfect pack

        if self.verbose {
            print!("Using ");
            // Multithreading only relevant for greedy solvers.
            if greedy && self.use_multithreading { print!("multithreaded"); }
            else if greedy { print!("single threaded"); }
            print!(" ");
            if greedy { print!("greedy packer"); }
            else { print!("exhaustive packer (slow)"); }
            println!(" for {}x{} on {} totems.", width, height, num_totems);
        }
        if !greedy {
            self.exhaustive.try_solve(width, height, bag)
        } else {
            // TODO use rectangle packing here
            if self.use_multithreading {
                multithread_solver!(self.greedy, width, height, bag)
            } else {
                self.greedy.try_solve(width, height, bag)
            }
        }
    }
}
