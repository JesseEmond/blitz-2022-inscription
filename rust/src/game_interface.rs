use crate::shape_info::ShapeVariant;
use serde::{Deserialize, Serialize};
use std::{iter, ops};

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

impl<T> ops::Index<Totem> for [T] {
    type Output = T;

    fn index(&self, index: Totem) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T> ops::Index<&Totem> for [T] {
    type Output = T;

    fn index(&self, index: &Totem) -> &Self::Output {
        &self[*index as usize]
    }
}

impl<T> ops::IndexMut<Totem> for [T] {
    fn index_mut(&mut self, index: Totem) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

impl<T> ops::IndexMut<&Totem> for [T] {
    fn index_mut(&mut self, index: &Totem) -> &mut Self::Output {
        &mut self[*index as usize]
    }
}

impl<T> ops::Index<Totem> for Vec<T> {
    type Output = T;

    fn index(&self, index: Totem) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T> ops::Index<&Totem> for Vec<T> {
    type Output = T;

    fn index(&self, index: &Totem) -> &Self::Output {
        &self[*index as usize]
    }
}

impl<T> ops::IndexMut<Totem> for Vec<T> {
    fn index_mut(&mut self, index: Totem) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

impl<T> ops::IndexMut<&Totem> for Vec<T> {
    fn index_mut(&mut self, index: &Totem) -> &mut Self::Output {
        &mut self[*index as usize]
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TotemQuestion {
    pub shape: Totem,
}

#[repr(transparent)]
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct TotemBag(pub [usize; TOTEM_COUNT]);

impl TotemBag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_iter<T, I>(src: T) -> Self
    where
        T: IntoIterator<Item = I>,
        TotemBag: ops::IndexMut<I, Output = usize>,
    {
        let mut slf = Self::new();
        for idx in src {
            slf[idx] += 1;
        }
        slf
    }

    pub fn total(&self) -> usize {
        return self.0.iter().sum();
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|&v| v == 0)
    }

    pub fn contains(&self, totem: &Totem) -> bool {
        self.0[totem] > 0
    }

    pub fn expand(&self) -> impl iter::Iterator<Item = Totem> + '_ {
        TOTEMS
            .iter()
            .flat_map(move |&t| iter::repeat(t).take(self.0[t]))
    }

    pub fn can_afford(&self, cost: &TotemBag) -> bool {
        let mut affordable = true;
        for totem in TOTEMS {
            affordable &= cost[totem] <= self[totem];
        }
        affordable
    }

    pub fn add(&mut self, cost: &TotemBag) {
        for totem in TOTEMS {
            self.0[totem] += cost[totem];
        }
    }

    pub fn subtract(&mut self, cost: &TotemBag) {
        for totem in TOTEMS {
            self.0[totem] -= cost[totem];
        }
    }

    pub fn min(&self) -> Totem {
        *TOTEMS.iter().min_by_key(|&t| self.0[t]).unwrap()
    }

    pub fn max(&self) -> Totem {
        *TOTEMS.iter().max_by_key(|&t| self.0[t]).unwrap()
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

impl ops::Index<&Totem> for TotemBag {
    type Output = usize;

    fn index(&self, index: &Totem) -> &Self::Output {
        &self.0[*index as usize]
    }
}

impl ops::IndexMut<Totem> for TotemBag {
    fn index_mut(&mut self, index: Totem) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

impl ops::IndexMut<&Totem> for TotemBag {
    fn index_mut(&mut self, index: &Totem) -> &mut Self::Output {
        &mut self.0[*index as usize]
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

pub type Point = (usize, usize);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TotemAnswer {
    pub shape: Totem,
    pub coordinates: [Point; 4],
}

impl TotemAnswer {
    pub fn new(shape: Totem, coordinates: [Point; 4]) -> Self {
        TotemAnswer { shape, coordinates }
    }

    pub fn offset_by(&self, x: usize, y: usize) -> TotemAnswer {
        let mut coords = self.coordinates.clone();
        for (dx, dy) in &mut coords {
            *dx += x;
            *dy += y;
        }
        TotemAnswer::new(self.shape, coords)
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
