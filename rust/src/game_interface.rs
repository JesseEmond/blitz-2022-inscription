use serde::{Deserialize, Serialize};

pub const TOTEM_COUNT: usize = 7;

#[repr(usize)]
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Hash, Copy, Clone, Debug)]
pub enum Totem {
    I = 0,
    J,
    L,
    O,
    S,
    T,
    Z,
}

impl Totem {
    pub fn iter() -> std::slice::Iter<'static, Totem> {
        static TOTEMS: [Totem; TOTEM_COUNT] = [
            Totem::I,
            Totem::J,
            Totem::L,
            Totem::O,
            Totem::S,
            Totem::T,
            Totem::Z,
        ];
        TOTEMS.iter()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TotemQuestion {
    pub shape: Totem,
}

pub type TotemBag = [usize; TOTEM_COUNT];

#[derive(Serialize, Deserialize, Debug)]
pub struct Question {
    pub totems: Vec<TotemQuestion>,
}

impl Question {
    pub fn get_totem_bag(&self) -> TotemBag {
        let mut bag = [0; TOTEM_COUNT];
        for totem in &self.totems {
            bag[totem.shape as usize] += 1;
        }
        bag
    }
}

pub type CoordinatePair = (usize, usize);

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
