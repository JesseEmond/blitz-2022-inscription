use serde::{Deserialize, Serialize};

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
        static TOTEMS: [Totem; 7] = [
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Question {
    pub totems: Vec<TotemQuestion>,
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

    pub fn offset_by(&self, x: usize, y: usize) -> TotemAnswer {
        let mut coords = self.coordinates.clone();
        for (dx, dy) in &mut coords {
            *dx += x;
            *dy += y;
        }
        TotemAnswer {
            shape: self.shape,
            coordinates: coords,
        }
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
