use crate::game_interface::{Answer, Question};

pub trait Solver {
    fn solve(question: &Question) -> Answer;
}
