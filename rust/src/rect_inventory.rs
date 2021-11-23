// Represents an inventory of rectangles made up of totem pieces.
// We precompute some offline and use them online to instead solve a "rectangle packing"
// problem for harder instances.

use crate::{
    dlx_solver,
    game_interface::{Question, TotemAnswer, TotemBag, TotemQuestion, TOTEMS},
    solver::Solver,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::io::{self, prelude::*};
use std::fs::{self, File};

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Dims {
    pub width: usize,
    pub height: usize,
}

impl Dims {
    fn new(width: usize, height: usize) -> Self {
        Dims { width, height }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Rectangle {
    pub dims: Dims,
    pub cost: TotemBag,
    pub totems: Vec<TotemAnswer>,
}

impl Rectangle {
    fn new(dims: Dims, cost: TotemBag, totems: Vec<TotemAnswer>) -> Self {
        Rectangle { dims, cost, totems }
    }

    // Rotates a rectangle 90 degrees clockwise.
    pub fn rotate(&self) -> Rectangle {
        // Flip width/height.
        let dims = Dims::new(/*width=*/self.dims.height, /*height=*/self.dims.width);  
        let mut totems = self.totems.clone();
        for totem in &mut totems {
            for coord in &mut totem.coordinates {
                let (x, y) = *coord;
                *coord = (y, self.dims.width - 1 - x);
            }
        }
        Rectangle::new(dims, self.cost.clone(), totems)
    }
}


/// Get all shape combinations that give full rectangles with area up to 'max_area'.
/// Rectangles are returned with (w, h) dimensions such that w <= h.
fn get_all_packings(max_area: usize) -> Vec<Rectangle> {
    let solver = dlx_solver::DlxSolver::new();
    let mut rectangles = Vec::new();
    for w in 1..=max_area {
        let max_h = max_area / w;
        for h in w..=max_h {
            if (w * h) % 4 != 0 {
                continue;
            }
            print!("  Finding shapes that make up {}x{} rectangles...  ", w, h);
            io::stdout().flush().unwrap();
            let num_shapes = w * h / 4;
            let mut num_found = 0;
            for totems in TOTEMS.iter().combinations_with_replacement(num_shapes) {
                let question = Question {
                    totems: totems.iter().map(|&t| TotemQuestion { shape: *t }).collect()
                };
                let bag = question.get_totem_bag();
                if let Some(sln) = solver.try_solve(w, h, &bag) {
                    num_found += 1;
                    rectangles.push(Rectangle {
                        dims: Dims { width: w, height: h },
                        cost: bag,
                        totems: sln,
                    });
                }
            }
            println!("{} found.", num_found);
        }
    }
    rectangles
}

// Lightweight representation of a rectangle, used for most of the heavy processing.
#[derive(Clone, Copy)]
pub struct RectangleMetadata {
    pub dims: Dims,
    pub index: usize,
}

impl RectangleMetadata {
    pub fn is_square(&self) -> bool {
        self.dims.width == self.dims.height
    }

    // Rotates the rectangle 90 degrees. Note that this requires later rotating the
    // matching rectangle as well!
    pub fn rotate(&mut self) {
        let (w, h) = (self.dims.width, self.dims.height);
        self.dims.width = h;
        self.dims.height = w;
    }
}

// This is what gets stored to disk.
type PrecomputedRectangles = Vec<Rectangle>;

#[derive(Clone)]
pub struct RectangleInventory {
    // List of all precomputed rectangles that can be made from Totem pieces.
    rectangles: PrecomputedRectangles,
    // Pointers to rectangle instances.
    pub metadata: Vec<RectangleMetadata>,
}

impl RectangleInventory {
    fn new(rectangles: &Vec<Rectangle>) -> Self {
        let mut metadata = Vec::with_capacity(rectangles.len());
        for (idx, rect) in rectangles.iter().enumerate() {
            let meta = RectangleMetadata { dims: rect.dims, index: idx };
            metadata.push(meta);
        }
        RectangleInventory { rectangles: rectangles.clone(), metadata: metadata }
    }

    pub fn from_precomputed(filename: &String) -> RectangleInventory {
        // TODO: make sure we can load.
        println!("Loading precomputed rectangle inventory...");
        let saved = match fs::read_to_string(filename) {
            Err(err) => {
                panic!("Failed to read precomputed rectangles at {}: {}. \
                       Did you run '--bin precompute_rects'?",
                       filename, err);
            }
            Ok(s) => s,
        };
        let rectangles: PrecomputedRectangles = serde_json::from_str(&saved).unwrap();
        println!("Loaded {} rectangles.", rectangles.len());
        Self::new(&rectangles)
    }

    pub fn from_scratch(max_area: usize) -> RectangleInventory {
        println!("Generating all rectangles that can be made up to area {}...", max_area);
        Self::new(&get_all_packings(max_area))
    }

    pub fn save(&self, filename: &std::path::Path) -> std::io::Result<()> {
        println!("Saving precomputed rectangles to {}...", filename.display());
        let json = serde_json::to_string(&self.rectangles).unwrap();
        let mut f = File::create(filename)?;
        f.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn get_cost(&self, metadata: &RectangleMetadata) -> &TotemBag {
        &self.rectangles[metadata.index].cost
    }

    // Returns all rectangles that could be used with the given bag.
    pub fn available_rectangles(&self, bag: &TotemBag) -> Vec<&RectangleMetadata> {
        self.metadata.iter().filter(|m| {
            bag.can_afford(&self.get_cost(m))
        }).collect()
    }

    pub fn get_rectangle(&self, metadata: &RectangleMetadata) -> &Rectangle {
        &self.rectangles[metadata.index]
    }
}