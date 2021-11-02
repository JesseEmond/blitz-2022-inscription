use crate::shape_info::ShapeVariant;
use serde::{Deserialize, Serialize};
use std::ops;

pub const TOTEM_COUNT: usize = 7;

pub const TOTEMS: [Totem; TOTEM_COUNT] = [
    Totem::I,
    Totem::J,
    Totem::L,
    Totem::O,
    Totem::S,
    Totem::T,
    Totem::Z,
];

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
    pub fn get_rotations(&self) -> &'static [ShapeVariant] {
        ShapeVariant::get_rotations(self)
    }
}

impl From<Totem> for usize {
    fn from(src: Totem) -> Self {
        src as usize
    }
}

impl From<usize> for Totem {
    fn from(src: usize) -> Self {
        assert!(src < TOTEM_COUNT);
        unsafe { std::mem::transmute(src) }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TotemQuestion {
    pub shape: Totem,
}

#[repr(transparent)]
#[derive(Default, Clone, Copy, Debug)]
pub struct TotemBag([usize; TOTEM_COUNT]);

impl TotemBag {
    pub fn from_iter<T, I>(src: T) -> Self
    where
        T: IntoIterator<Item = I>,
        TotemBag: ops::IndexMut<I, Output = usize>,
    {
        let mut slf = Self::default();
        for idx in src {
            slf[idx] += 1;
        }
        slf
    }
}

impl ops::Index<usize> for TotemBag {
    type Output = usize;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl ops::IndexMut<usize> for TotemBag {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl ops::Index<Totem> for TotemBag {
    type Output = usize;

    fn index(&self, index: Totem) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl ops::IndexMut<Totem> for TotemBag {
    fn index_mut(&mut self, index: Totem) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Question {
    pub totems: Vec<TotemQuestion>,
}

impl Question {
    pub fn get_totem_bag(&self) -> TotemBag {
        TotemBag::from_iter(self.totems.iter().map(|t| t.shape))
    }
}

pub type CoordinatePair = (usize, usize);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TotemAnswer {
    pub shape: Totem,
    pub coordinates: [CoordinatePair; 4],
}

impl TotemAnswer {
    pub fn new(shape: Totem, coordinates: [CoordinatePair; 4]) -> Self {
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
