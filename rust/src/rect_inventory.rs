// Pre-comptued rectangle inventory made up of Tetris pieces.
use crate::{
    dlx_solver,
    game_interface::{Totem, TotemAnswer},
    shape_info::ShapeDist,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::fs::File;

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
    pub cost: ShapeDist,
    pub totems: Vec<TotemAnswer>,
}

impl Rectangle {
    fn new(dims: Dims, cost: ShapeDist, totems: Vec<TotemAnswer>) -> Self {
        Rectangle { dims, cost, totems }
    }

    pub fn rotate(&self) -> Rectangle {
        let dims = Dims::new(/*width=*/self.dims.height, /*height=*/self.dims.width);  // flip width/height
        let mut totems = self.totems.clone();
        for totem in &mut totems {
            for coord in &mut totem.coordinates {
                let (x, y) = *coord;
                coord.0 = y;
                coord.1 = self.dims.width - 1 - x;
            }
        }
        Rectangle::new(dims, self.cost, totems)
    }
}


/// Get all shape combinations that give full rectangles with area up to 'max_area'.
/// Rectangles are returned with (w, h) dimensions such that w <= h.
fn get_all_packings(max_area: usize) -> Vec<Rectangle> {
    let mut rectangles = Vec::new();
    for w in 1..=max_area {
        let max_h = max_area / w;
        for h in w..=max_h {
            if (w * h) % 4 != 0 {
                continue;
            }
            println!("  Finding shapes that make up {}x{} rectangles...", w, h);
            let num_shapes = w * h / 4;
            for totems in Totem::iter().combinations_with_replacement(num_shapes) {
                let mut dist = [0; 7];
                for totem in totems {
                    dist[*totem as usize] += 1;
                }
                if let Some(sln) = dlx_solver::try_fit(w, h, &dist) {
                    rectangles.push(Rectangle {
                        dims: Dims { width: w, height: h },
                        cost: dist,
                        totems: sln,
                    });
                }
            }
        }
    }
    rectangles
}

#[derive(Clone, Copy)]
pub struct RectangleMetadata {
    pub dims: Dims,
    pub index: usize,
}

impl RectangleMetadata {
    pub fn is_square(&self) -> bool {
        self.dims.width == self.dims.height
    }

    pub fn rotate(&mut self) {
        let (w, h) = (self.dims.width, self.dims.height);
        self.dims.width = h;
        self.dims.height = w;
    }
}

type PrecomputedRectangles = Vec<Rectangle>;

#[derive(Clone)]
pub struct RectangleInventory {
    // List of all precomputed rectangles that can be made from Totem pieces.
    rectangles: PrecomputedRectangles,
    // Pointers to rectangle instances.
    pub metadata: Vec<RectangleMetadata>,
}

pub fn can_afford(price: &ShapeDist, bag: &ShapeDist) -> bool {
    let mut affordable = true;
    for i in 0..7 {
        affordable &= price[i] <= bag[i];
    }
    affordable
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

    pub fn from_precomputed() -> RectangleInventory {
        println!("Loading precomputed rectangle inventory...");
        let saved = include_str!("precomputed_area_32.rects");  // If this fails, you need to precompute rectangles.
        let rectangles: PrecomputedRectangles = serde_json::from_str(&saved).unwrap();
        println!("Loaded {} rectangles.", rectangles.len());
        Self::new(&rectangles)
    }

    #[allow(dead_code)]
    pub fn from_scratch(max_area: usize) -> RectangleInventory {
        println!("Generating all rectangles that can be made up to area {}...", max_area);
        Self::new(&get_all_packings(max_area))
    }

    #[allow(dead_code)]
    pub fn save(&self, filename: &std::path::Path) -> std::io::Result<()> {
        println!("Saving precomputed rectangles to {}...", filename.display());
        let json = serde_json::to_string(&self.rectangles).unwrap();
        let mut f = File::create(filename)?;
        f.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn get_cost(&self, metadata: &RectangleMetadata) -> &ShapeDist {
        &self.rectangles[metadata.index].cost
    }

    pub fn available_rectangles(&self, bag: &ShapeDist) -> Vec<&RectangleMetadata> {
        // TODO return iterator...?
        self.metadata.iter().filter(|m| {
            can_afford(&self.get_cost(m), bag)
        }).collect()
    }

    pub fn get_rectangle(&self, metadata: &RectangleMetadata) -> &Rectangle {
        &self.rectangles[metadata.index]
    }
}