// Evaluate the perfect packing probability of a solver at a given level.
// Useful to compare methods and to do performance profiling.

extern crate application;

use application::{
    game_interface::{Question, TotemQuestion, TOTEMS},
    hybrid_solver,
    solver::Solver,
    scoring::{score, OptimalDimensions},
};
use clap::{Arg, App};
use rand::seq::SliceRandom;

// Imports only for visualization, when enabled.
#[cfg(feature = "visualize")]
use application::{
    game_interface::Answer,
    solver,
};

type SelectedSolver = hybrid_solver::HybridSolver;

// Returns 95% confidence interval for the success probability given a given amount of 'successes'
// over a given amount of 'trials'.
// https://en.wikipedia.org/wiki/Binomial_proportion_confidence_interval
// Using the Wilson score interval.
fn binomial_confidence_interval(successes: u64, trials: u64) -> (f64, f64) {
    let n = trials as f64;
    let p_hat = successes as f64 / n;
    let z: f64 = 1.96;  // alpha = 1 - 0.95 = 0.05 for 95% confidence, 1-alpha/2=0.975 => z=1.96
    // Give names to variables for the general form of a +- b * c.sqrt(), to make it (a bit) easier to follow.
    let a = (p_hat + z * z / (2f64 * n)) / (1f64 + z * z / n);
    let b = z / (1f64 + z * z / n);
    let c = p_hat * (1f64 - p_hat) / n + z * z / (4f64 * n * n);
    let lower = a - b * c.sqrt();
    (if lower >= 0f64 { lower } else { 0f64 }, a + b * c.sqrt())
}

fn debug_packing_probability(level: usize, solver: &SelectedSolver) {
    let num_totems = 1 << level;
    let mut rng = rand::thread_rng();
    let mut total_runs = 0;
    let mut perfect_packs = 0;
    let mut perfect_pack_seconds = 0f64;
    let mut last_time: std::time::Instant = std::time::Instant::now();
    let start_time = std::time::Instant::now();

    let optimal_dims = OptimalDimensions::new();
    let (w, h) = optimal_dims.level_dims(level).first().unwrap();
    println!("Searching for perfect packs for level {}, with {} totems. Need to pack {}x{} for a score of {}.",
             level + 1, num_totems, *w, *h, score(num_totems, *w, *h));
    loop {
        let attempt_time = std::time::Instant::now();
        let question = Question {
            totems: (0..num_totems).map(
                |_| TotemQuestion { shape: *TOTEMS.choose(&mut rng).unwrap() }).collect()
        };
        let bag = question.get_totem_bag();
        if let Some(_sln) = solver.try_solve(*w, *h, &bag) {
            perfect_packs += 1;
            perfect_pack_seconds += attempt_time.elapsed().as_secs_f64();
            #[cfg(feature = "visualize")]
            solver::visualize(&Answer { totems: _sln });  // To visually make sure the solutions are valid.
        }
        total_runs += 1;
        if last_time.elapsed().as_secs_f64() > 0.5 {
            let total_time = start_time.elapsed().as_secs_f64();
            let pack_speed = (total_runs as f64) / total_time;
            let pack_ratio = (perfect_packs as f32) / (total_runs as f32);
            let (lower_bound, upper_bound) = binomial_confidence_interval(perfect_packs, total_runs);
            let secs_per_success = perfect_pack_seconds / (perfect_packs as f64);
            println!("{} / {} perfect packs (p={:.1}%   alpha=0.05 interval=[{:.1}%, {:.1}%]),   ~{:.2}s/it   ~{:.2}s/success its",
                     perfect_packs, total_runs, pack_ratio * 100f32, lower_bound * 100f64, upper_bound * 100f64,
                     1f64 / pack_speed, secs_per_success);
            last_time = std::time::Instant::now();
        }
    }
}

fn is_valid_level(level: String) -> Result<(), String> {
    let level: usize = match level.parse() {
        Ok(level) => level,
        Err(_) => {
            return Err(String::from("level must be a positive integer"));
        }
    };
    if level < 1 || level > 10 {
        Err(String::from("level must be between 1 and 10, inclusively"))
    } else {
        Ok(())
    }
}


fn main() {
    let matches = App::new("Coveo 2022 Inscription Evaluation")
                          .arg(Arg::with_name("level")
                               .value_name("LEVEL")
                               .long("level")
                               .help("Level that we are evaluating on")
                               .required(true)
                               .validator(is_valid_level))
                          .get_matches();
    let level = matches.value_of("level").unwrap();
    let level: usize = level.parse().unwrap();
    let level = level - 1;  // Logic assumes that levels are 0-indexed.

    let solver = SelectedSolver::with_options(/*multithreading=*/true, /*verbose=*/false);
    debug_packing_probability(level, &solver);
}