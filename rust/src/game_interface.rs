use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Totem {
    I,
    O,
    J,
    L,
    S,
    Z,
    T,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TotemQuestion {
    pub shape: Totem,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Question {
    pub totems: Vec<TotemQuestion>,
}

pub type CoordinatePair = (i32, i32);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TotemAnswer {
    pub shape: Totem,
    pub coordinates: Vec<CoordinatePair>,
}

impl TotemAnswer {
    pub fn new(shape: Totem, coordinates: Vec<CoordinatePair>) -> Self {
        TotemAnswer { shape, coordinates }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Answer {
    pub totems: Vec<TotemAnswer>,
}

impl Answer {
    pub fn new(totems: Vec<TotemAnswer>) -> Self {
        Answer { totems }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameMessage {
    pub tick: i32,
    pub payload: Question,
}