// Dancing-Links X Algorithm solver for an "exact cover" reformulation of our problem.
// Note that this only works for problems that require a perfect packing (4*totems == width*height)
// and is too slow for larger problems (e.g. > 8 totems).

// This uses Donald Knuth's X Algorithm, with a "Dancing Links" internal representation,
// to solve an exact cover problem: https://arxiv.org/pdf/cs/0011047.pdf

// The formulation of this problem as an exact cover matrix is inspired by:
// https://toddtaomae.wordpress.com/2014/08/07/solving-the-tetris-cube/
// We create a column for each (x, y) position to fill, and for each totem that we must place.
// We create a row for every possible (x, y) placement of a given totem rotation.

use crate::{
    game_interface::{Totem, TotemAnswer, TotemBag},
    shape_info::ShapeVariant,
    solver::Solver,
};
use dlx;

// What is the nth totem in our bag, assuming we put them all in order?
fn nth_totem(bag: &TotemBag, index: usize) -> Option<Totem> {
    bag.expand().nth(index)
}

// Totem board, used to generate rows for an exact cover problem covering all possible (x, y)
// placements of each rotation of each totem.
struct TotemBoardCoverRows {
    width: usize,
    height: usize,

    // iteration variables
    x: usize,
    y: usize,
    bag: TotemBag,
    totem_index: usize,
    rotation_idx: usize,
}

impl TotemBoardCoverRows {
    fn new(width: usize, height: usize, bag: &TotemBag) -> Self {
        TotemBoardCoverRows {
            width: width, height: height,
            x: 0, y: 0,
            bag: bag.clone(),
            totem_index: 0,
            rotation_idx: 0,
        }
    }

    fn current_rotation(&self) -> Option<&'static ShapeVariant> {
        let totem = nth_totem(&self.bag, self.totem_index)?;
        Some(&ShapeVariant::get_rotations(&totem)[self.rotation_idx])
    }

    fn next_rotation(&mut self) -> Option<&'static ShapeVariant> {
        let totem = nth_totem(&self.bag, self.totem_index)?;
        let rotations = ShapeVariant::get_rotations(&totem);
        self.rotation_idx += 1;
        if self.rotation_idx >= rotations.len() {
            self.rotation_idx = 0;
            self.totem_index += 1;
        }
        self.current_rotation()
    }
}

impl Iterator for TotemBoardCoverRows {
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
    // Index of this totem within the TotemBag distribution.
    totem_index: usize,
}

// Encode (x,y) coordinates into a cover row index.
fn coord_to_index(coords: (usize, usize), width: usize) -> dlx::Index {
    let (x, y) = coords;
    return y * width + x;
}

// Extracts the encoded (x,y) coordinates from a cover row index.
fn index_to_coord(index: dlx::Index, width: usize) -> (usize, usize) {
    let x = index % width;
    let y = index / width;
    (x, y)
}

// Row for this totem placement. Has '1's in each coordinate of the totem and in the slot for
// this totem's index in our bag.
fn totem_cover_row(placement: &Placement, width: usize, height: usize) -> Vec<dlx::Index> {
    let variant = placement.variant.offset_by(placement.x, placement.y);
    let coords = variant.coords;
    vec![
        coord_to_index(coords[0], width),
        coord_to_index(coords[1], width),
        coord_to_index(coords[2], width),
        coord_to_index(coords[3], width),
        // totem indices are after the w*h coordinates.
        width * height + placement.totem_index,
    ]
}

struct Solutions {
    width: usize,
    height: usize,
    bag: TotemBag,
    totems: Option<Vec<TotemAnswer>>,
}

impl dlx::Solutions for Solutions {
    fn push(&mut self, sol: dlx::Solution) -> bool {
        let mut totems = Vec::new();
        for row in sol {
            let coords = [
                index_to_coord(row[0], self.width),
                index_to_coord(row[1], self.width),
                index_to_coord(row[2], self.width),
                index_to_coord(row[3], self.width),
            ];
            let totem_index = row[4] - self.width * self.height;
            let totem = nth_totem(&self.bag, totem_index).unwrap();
            totems.push(TotemAnswer { coordinates: coords, shape: totem });
        }
        self.totems = Some(totems);
        false  // stop after the first solution
    }
}

pub struct DlxSolver {
}

impl Solver for DlxSolver {
    fn new() -> Self {
        Self { }
    }

    fn try_solve(&self, width: usize, height: usize, bag: &TotemBag) -> Option<Vec<TotemAnswer>> {
        let num_totems = bag.total();
        let num_squares = width * height;
        assert!(num_totems * 4 == num_squares, "DLX solver only works for exact fits.");
        let num_columns = num_squares + num_totems;
        let mut solver = dlx::Solver::new(num_columns, TotemBoardCoverRows::new(width, height, bag));
        let mut sols = Solutions { totems: None, width: width, height: height, bag: bag.clone() };
        solver.solve(Vec::new(), &mut sols);
        sols.totems
    }
}