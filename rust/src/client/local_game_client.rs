use application::{
    game_interface::{GameMessage, Question, Totem, TotemQuestion},
    solver::Solver,
};
use rand::{
    self,
    distributions::{Distribution, Uniform},
};
use std::{env, marker::PhantomData};

pub struct LocalGameClient<S>
where
    S: Solver,
{
    _solver: PhantomData<S>,
}

impl<S> LocalGameClient<S>
where
    S: Solver,
{
    pub fn new() -> Self {
        Self {
            _solver: PhantomData,
        }
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
            let idx: usize = die.sample(&mut rng);
            let shape: Totem = unsafe { std::mem::transmute(idx) };
            questions.push(TotemQuestion { shape });
        }
        let question = Question { totems: questions };
        let game_message = GameMessage {
            tick: 1,
            payload: question,
        };

        S::solve(&game_message.payload);
    }
}
