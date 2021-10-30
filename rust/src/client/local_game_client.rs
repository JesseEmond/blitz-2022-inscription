use std::env;
use rand;
use rand::distributions::{Distribution, Uniform};
use crate::solver::Solver;
use crate::game_interface::{GameMessage, Question, Totem, TotemQuestion};

pub struct LocalGameClient {
    solver: Solver,
}

impl LocalGameClient {
    pub fn new(solver: Solver) -> Self {
        LocalGameClient { solver }
    }

    pub async fn run(&self) {
        println!("[Running in local mode]");

        let mut rng = rand::thread_rng();
        let n_totems = match env::var("TOTEMS") {
            Ok(val) => val.parse().unwrap(),
            Err(_) => 8,
        };
        let mut questions: Vec<TotemQuestion> = Vec::new();
        let die = Uniform::from(0..7);
        for _ in 0..n_totems {
            let idx = die.sample(&mut rng);
            let totem = [Totem::I, Totem::J, Totem::L, Totem::O, Totem::T, Totem::S, Totem::Z][idx];
            questions.push(TotemQuestion { shape: totem });
        }
        let question = Question { totems: questions };
        let game_message = GameMessage { tick: 1, payload: question };

        self.solver.get_answer(&game_message).expect("There was an error in the solver's code!");
    }
}
