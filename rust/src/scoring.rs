use ordered_float::OrderedFloat;
use std::cmp;
use std::collections::HashSet;
use std::iter::FromIterator;

pub fn score(num_totems: usize, width: usize, height: usize) -> f32 {
    (10 * num_totems as i32 - width as i32 * height as i32) as f32 * cmp::min(width, height) as f32
        / cmp::max(width, height) as f32
}

// width, height
pub type Dims = (usize, usize);

// Gives a list of all dimensions that can cover num_totems; squares and rectangles.
// Also provide dimensions that cover much more than num_totems, in case we can't find
// a fit in the tighter dimensions.
// This list has no meaningful ordering, it should be further sorted by score.
fn get_all_dims(num_totems: usize) -> Vec<Dims> {
    let n_squares = num_totems * 4;
    let optimal_square_side: usize = (n_squares as f64).sqrt().ceil() as usize;
    // Assume that we'll be able to fit in a square with a side twice as big. This is a bit overkill,
    // but we won't actually try to fit all of them. If we can't fit the totems in a square 4x as big
    // as the optimal one, we have other problems.
    let max_square_side = std::cmp::max(optimal_square_side * 2, 4); // Ensure we can fit the 4x1 totem.
    let mut dims = HashSet::with_capacity(max_square_side * 2);
    for len in 1..(max_square_side + 1) {
        if len * len >= n_squares {
            dims.insert((len, len));
        }
        let other_side = (n_squares as f64 / len as f64).ceil() as usize;
        assert!(len * other_side >= n_squares);
        dims.insert((len, other_side));
    }
    Vec::from_iter(dims)
}

// Helper to get an optimal list of dimensions for a given number of totems, to maximize score.
pub struct OptimalDimensions {
    // Ordered list of dimensions to consider, per level.
    level_dims: Vec<Vec<Dims>>,
}

impl OptimalDimensions {
    pub fn new() -> Self {
        let level_dims = (0..10_usize)
            .into_iter()
            .map(|level| {
                let num_totems = 1 << level;
                let mut all_dims = get_all_dims(num_totems);
                all_dims
                    .sort_by_key(|(w, h)| cmp::Reverse(OrderedFloat(score(num_totems, *w, *h))));
                all_dims
            })
            .collect();
        OptimalDimensions { level_dims }
    }

    // Get the list of optimal dimensions for a given level (NOTE: 0-indexed!).
    pub fn level_dims(&self, level: usize) -> &[Dims] {
        &self.level_dims[level]
    }
}
