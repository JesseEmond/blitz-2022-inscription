// Tool that runs the 10 levels over and over until we hit a perfect score.

extern crate application;

use application::{
    game_interface::{Answer, Question, Totem, TotemBag, TotemQuestion, TOTEMS},
    hybrid_solver,
    solver::Solver,
    scoring::{score, OptimalDimensions},
};
use rand::seq::SliceRandom;
use std::{
    collections::HashSet,
    time::Instant,
};

type SelectedSolver = hybrid_solver::HybridSolver;

fn level_optimal_score(level: usize, optimal_dims: &OptimalDimensions) -> f32 {
    if level == 0 {
        1.5  // Best that can be done for "I" (always) is 1x4.
    } else {
        let num_totems = 1 << level;
        let (w, h) = optimal_dims.level_dims(level).first().unwrap();
        score(num_totems, *w, *h)
    }
}

fn is_valid_solution(question: &Question, answer: &Answer, start_time: &Instant) -> bool {
    let seconds = start_time.elapsed().as_secs_f64();
    if seconds >= 1f64 {
        println!("[!] {{Took more than 1s}}  Took {:?} to solve.", seconds);
        return false;
    }
    let mut totems = TotemBag::new();
    let mut coords = HashSet::with_capacity(question.totems.len() * 4);
    for totem in answer.totems.iter() {
        for coord in totem.coordinates {
            if !coords.insert(coord) {
                println!("[!] {{Dupe coords}}  {:?} appears more than once.", coord);
                return false;
            }
        }
        totems[totem.shape] += 1;
    }
    if !coords.contains(&(0, 0)) {
        println!("[!] {{Missing (0,0)}}  (0,0) missing from answer.");
        return false;
    }
    if totems.0 != question.get_totem_bag().0 {
        println!("[!] {{Incorrect shapes}}  Shapes mismatch. Got {:?}, expected {:?}.",
                 totems, question.get_totem_bag());
        return false;
    }
    true
}

fn generate_level(level: usize) -> Question {
    if level == 0 {  // Level 1 is always "I".
        return Question { totems: vec![TotemQuestion { shape: Totem::I }] };
    }
    let mut rng = rand::thread_rng();
    let num_totems = 1 << level;
    Question {
        totems: (0..num_totems).map(
            |_| TotemQuestion { shape: *TOTEMS.choose(&mut rng).unwrap() }).collect()
    }
}

// Generates and solves a given level (0-indexed).
// Returns our score if our solution was valid _and_ optimal.
fn run_level(solver: &SelectedSolver, level: usize) -> Option<f32> {
    let question = generate_level(level);
    let start_time = Instant::now();
    let answer = solver.solve(&question);
    if !is_valid_solution(&question, &answer, &start_time) {
        None
    } else {
        let max_x = answer
            .totems
            .iter()
            .flat_map(|t| t.coordinates.iter().map(|p| p.0))
            .max()
            .unwrap();
        let max_y = answer
            .totems
            .iter()
            .flat_map(|t| t.coordinates.iter().map(|p| p.1))
            .max()
            .unwrap();
        let w = max_x + 1;
        let h = max_y + 1;
        Some(score(question.totems.len(), w, h))
    }
}

// Runs up to a full round of 10 levels (early exits if an invalid or suboptimal
// solution is found).
// Returns the total score of the rounds, if they were all optimal.
fn run_round(solver: &SelectedSolver, optimal_dims: &OptimalDimensions) -> Option<f32> {
    let mut total_score = 0f32;
    for level in 0..10 {
        let score = run_level(solver, level)?;
        let optimal_score = level_optimal_score(level, optimal_dims);
        if (optimal_score - score).abs() > 1.0e-5 {
            println!("Suboptimal level {}  ({} < {}), abort round.", level+1, score, optimal_score);
            return None;
        }
        total_score += score;
    }
    Some(total_score)
}


fn main() {
    let optimal_dims = OptimalDimensions::new();
    let solver = SelectedSolver::new();
    let mut round = 0;
    loop {
        println!("Round #{}", round+1);
        if let Some(score) = run_round(&solver, &optimal_dims) {
            println!("Round score: {}", score);
            println!("That's optimal! Found after {} rounds.", round+1);
            break;
        }
        println!("\n\n");
        round += 1;
    }
}