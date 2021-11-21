use crate::{
    game_interface::{Totem, TotemAnswer},
    shape_info::{ShapeDist, ShapeVariant},
};
use dlx;

fn nth_totem(dist: &ShapeDist, index: usize) -> Option<Totem> {
    let mut index = index;
    for totem in Totem::iter() {
        let n_totems = dist[*totem as usize];
        if index < n_totems {
            return Some(*totem)
        }
        index -= n_totems;
    }
    None
}

// Totem board, used to generate rows for an exact cover problem covering all possible (x, y)
// placements of each rotation of each totem.
struct TotemBoard {
    width: usize,
    height: usize,

    // iteration variables
    x: usize,
    y: usize,
    dist: ShapeDist,
    totem_index: usize,
    rotation_idx: usize,
}

impl TotemBoard {
    fn new(width: usize, height: usize, dist: &ShapeDist) -> TotemBoard {
        TotemBoard {
            width: width, height: height,
            x: 0, y: 0,
            dist: *dist,
            totem_index: 0,
            rotation_idx: 0,
        }
    }

    fn current_rotation(&self) -> Option<&'static ShapeVariant> {
        let totem = nth_totem(&self.dist, self.totem_index)?;
        Some(&ShapeVariant::get_rotations(&totem)[self.rotation_idx])
    }

    fn next_rotation(&mut self) -> Option<&'static ShapeVariant> {
        let totem = nth_totem(&self.dist, self.totem_index)?;
        let rotations = ShapeVariant::get_rotations(&totem);
        self.rotation_idx += 1;
        if self.rotation_idx >= rotations.len() {
            self.rotation_idx = 0;
            self.totem_index += 1;
        }
        self.current_rotation()
    }
}

impl Iterator for TotemBoard {
    type Item = dlx::Row;

    fn next(&mut self) -> Option<Self::Item> {
        let mut variant = self.current_rotation()?;
        while (variant.width as usize) > self.width || (variant.height as usize) > self.height {
            variant = self.next_rotation()?;
        }
        let placement = Placement {
            x: self.x,
            y: self.y,
            variant: variant,
            totem_index: self.totem_index,
        };
        let row = totem_cover_row(&placement, self.width, self.height);
        self.x += 1;
        if self.x + variant.width as usize > self.width {
            self.x = 0;
            self.y += 1;
            if self.y + variant.height as usize > self.height {
                self.y = 0;
                self.next_rotation();  // Note: ignore the result, this is handled on the next iteration.
            }
        }
        Some(row)
    }
}

struct Placement {
    // (x, y) of the bottom left corner of the shape
    x: usize,
    y: usize,
    // Rotation variant of the totem.
    variant: &'static ShapeVariant,
    // Index of this totem within the ShapeDist distribution.
    totem_index: usize,
}

fn coord_to_index(coords: (usize, usize), width: usize) -> dlx::Index {
    let (x, y) = coords;
    return y * width + x;
}

fn index_to_coord(index: dlx::Index, width: usize) -> (usize, usize) {
    let x = index % width;
    let y = index / width;
    (x, y)
}

fn totem_cover_row(placement: &Placement, width: usize, height: usize) -> Vec<dlx::Index> {
    let variant = placement.variant.offset_by(placement.x, placement.y);
    let coords = variant.coords;
    vec![
        coord_to_index(coords[0], width),
        coord_to_index(coords[1], width),
        coord_to_index(coords[2], width),
        coord_to_index(coords[3], width),
        width * height + placement.totem_index,
    ]
}

struct Solutions {
    width: usize,
    height: usize,
    dist: ShapeDist,
    totems: Option<Vec<TotemAnswer>>,
}

impl dlx::Solutions for Solutions {
    fn push(&mut self, sol: dlx::Solution) -> bool {
        // TODO convert sln to TotemAnswer, store in sol.totems
        let mut totems = Vec::new();
        for row in sol {
            let coords = vec![
                index_to_coord(row[0], self.width),
                index_to_coord(row[1], self.width),
                index_to_coord(row[2], self.width),
                index_to_coord(row[3], self.width),
            ];
            let totem_index = row[4] - self.width * self.height;
            let totem = nth_totem(&self.dist, totem_index).unwrap();
            totems.push(TotemAnswer { coordinates: coords, shape: totem });
        }
        self.totems = Some(totems);
        false  // stop after the first solution
    }
}

pub fn try_fit(width: usize, height: usize, dist: &ShapeDist) -> Option<Vec<TotemAnswer>> {
    let num_totems: usize = dist.iter().sum();
    let num_squares = width * height;
    let num_columns = num_squares + num_totems;
    let mut solver = dlx::Solver::new(num_columns, TotemBoard::new(width, height, dist));
    let mut sols = Solutions { totems: None, width: width, height: height, dist: *dist };
    solver.solve(Vec::new(), &mut sols);
    sols.totems
}