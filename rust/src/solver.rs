use crate::{
    game_interface::{Answer, GameMessage, Question, Totem, TotemAnswer},
    scoring::{score, Dims, OptimalDimensions},
    shape_info::ShapeVariant,
};
use std::{error::Error, cmp, time::Instant};

struct Board {
    width: usize,
    height: usize,
    grid: Vec<Vec<bool>>,
    touchpoints: Vec<Vec<u32>>,
    totems: Vec<TotemAnswer>,
}

impl Board {
    fn new(width: usize, height: usize, answer_size: usize) -> Board {
        let grid = vec![vec![false; width]; height];
        let mut touchpoints = vec![vec![0; width]; height];
        // Treat borders as touchpoints
        for x in 0..width {
            touchpoints[0][x] += 1;
            touchpoints[height - 1][x] += 1;
        }
        for y in 0..height {
            touchpoints[y][0] += 1;
            touchpoints[y][width - 1] += 1;
        }
        touchpoints[0][0] += 1; // Give (0,0) a boost to ensure we set it.
                                // TODO "smallest unset at x"
        Board {
            width,
            height,
            grid,
            touchpoints,
            totems: Vec::with_capacity(answer_size),
        }
    }

    fn mark(&mut self, shape: &ShapeVariant) {
        for (x, y) in &shape.coords {
            self.grid[*y][*x] = true;
            // TODO: update "smallest unset at x"
            if *y > 0 {
                self.touchpoints[*y - 1][*x] += 1;
            }
            if *y + 1 < self.height {
                self.touchpoints[*y + 1][*x] += 1;
            }
            if *x > 0 {
                self.touchpoints[*y][*x - 1] += 1;
            }
            if *x + 1 < self.width {
                self.touchpoints[*y][*x + 1] += 1;
            }
        }
        self.totems
            .push(TotemAnswer::new(shape.shape, shape.coords.to_vec()));
    }

    fn fits(&self, shape: &ShapeVariant) -> Option<bool> {
        for (x, y) in &shape.coords {
            if *x >= self.width || *y >= self.height {
                return None;
            }
            if self.grid[*y][*x] {
                return Some(false);
            }
        }
        Some(true)
    }

    fn num_touchpoints(&self, shape: &ShapeVariant) -> u32 {
        let mut total = 0;
        for (x, y) in &shape.coords {
            total += self.touchpoints[*y][*x];
        }
        total
    }

    fn move_first_fit_above(&self, shape: &mut ShapeVariant) -> bool {
        loop {
            match self.fits(shape) {
                Some(fitted) => {
                    if fitted {
                        return true;
                    } else {
                        for point in &mut shape.coords {
                            point.1 += 1;
                        }
                    }
                }
                None => return false,
            }
        }
    }
}

type ShapeDist = [usize; 7];

fn try_gravity_greedy_fit(board: &mut Board, mut dist: ShapeDist) -> Option<Vec<TotemAnswer>> {
    loop {
        let mut best_shape: Option<ShapeVariant> = None;
        let mut best_touchpoints: u32 = 0;
        let mut shapes_left = 0;
        for totem in Totem::iter() {
            let n_totem = dist[*totem as usize];
            shapes_left += n_totem;
            if n_totem > 0 {
                for variant in ShapeVariant::get_rotations(totem) {
                    for dx in 0..(board.width as i32 + 1 - variant.width as i32) {
                        let mut variant = variant.clone();
                        for point in &mut variant.coords {
                            point.0 += dx as usize;
                        }
                        if board.move_first_fit_above(&mut variant) {
                            let touchpoints = board.num_touchpoints(&variant);
                            if touchpoints > best_touchpoints {
                                best_touchpoints = touchpoints;
                                best_shape = Some(variant);
                            }
                        }
                    }
                }
            }
        }
        if shapes_left == 0 {
            return Some(board.totems.clone());
        }
        match best_shape {
            Some(shape) => {
                board.mark(&shape);
                dist[shape.shape as usize] -= 1;
            }
            None => return None,
        }
    }
}

fn min_dimensions_needed(dist: &ShapeDist) -> Dims {
    let mut dims = (0, 0);
    for totem in Totem::iter() {
        if dist[*totem as usize] > 0 {
            let dim = ShapeVariant::minimum_dims(totem);
            dims.0 = cmp::max(dims.0, dim.0);
            dims.1 = cmp::max(dims.1, dim.1);
        }
    }
    dims
}

fn solve_greedy(question: &Question, level: usize,
                optimal_dims: &OptimalDimensions) -> Vec<TotemAnswer> {
    let dist = get_shape_distribution(question);
    let min_dims = min_dimensions_needed(&dist);
    let answer_size = question.totems.len();
    for (w, h) in optimal_dims.level_dims(level) {
        // Note: implicit assumption here that optimal_dims have the shortest dim first,
        // and that min_dimensions_needed does so based on minimal width
        if min_dims.0 > *w || min_dims.1 > *h {
            println!("Skipping {}x{} (would have given {}), could not fit {}x{} totem",
                     *w, *h, score(answer_size, *w, *h), min_dims.0, min_dims.1);
            continue;
        }
        println!("Trying {}x{}... would give {}", *w, *h, score(answer_size, *w, *h));
        if let Some(fit) = try_gravity_greedy_fit(&mut Board::new(*w, *h, answer_size), dist) {
            return fit;
        }
    }
    println!("!!! FAILED TO FIND SOLUTION. Should increase ranges in 'optimal dims'.");
    // If we can't find a solution with a 4*totem x 4*totem grid....
    try_gravity_greedy_fit(&mut Board::new(4*answer_size, 4*answer_size, answer_size), dist).unwrap()
}

#[cfg(feature = "visualize")]
fn visualize(answer: &[TotemAnswer]) {
    static GLYPHS: [char; 7] = ['I', 'J', 'L', 'O', 'S', 'T', 'Z'];
    let mut max_x = 0;
    let mut max_y = 0;
    for totem in answer {
        for (x, y) in &totem.coordinates {
            max_x = std::cmp::max(max_x, *x);
            max_y = std::cmp::max(max_y, *y);
        }
    }
    let w = max_x + 1;
    let h = max_y + 1;
    let mut lines = vec![vec!['.'; w as usize]; h as usize];
    for totem in answer {
        for (x, y) in &totem.coordinates {
            lines[*y][*x] = GLYPHS[totem.shape as usize];
        }
    }
    for line in lines.iter().rev() {
        for c in line {
            print!("{}", c);
        }
        println!();
    }
    println!("{}x{}, score={}", w, h, score(answer.len(), w, h));

}

fn get_shape_distribution(question: &Question) -> ShapeDist {
    let mut dist: ShapeDist = [0; 7];
    for totem in &question.totems {
        dist[totem.shape as usize] += 1;
    }
    dist
}

pub struct Solver {
    optimal_dims: OptimalDimensions,
}

impl Solver {
    /// Initialize your solver
    pub fn new() -> Self {
        Solver { optimal_dims: OptimalDimensions::new() }
    }

    /// Answer the question
    pub fn get_answer(&self, game_message: &GameMessage) -> Result<Answer, Box<dyn Error>> {
        let question = &game_message.payload;
        let num_totems = question.totems.len();
        println!("Received question with {} totems.", num_totems);

        let inferred_level = (num_totems as f64).log2().ceil() as usize;
        let (optimal_w, optimal_h) = self.optimal_dims.level_dims(inferred_level).next().unwrap();
        println!("Optimal dims for level {} would be {}x{}, which would give score {}",
                 inferred_level + 1, optimal_w, optimal_h, score(num_totems, *optimal_w, *optimal_h));

        #[cfg(feature = "timing")]
        let now = Instant::now();

        let solution = solve_greedy(question, inferred_level, &self.optimal_dims);


        #[cfg(feature = "visualize")]
        visualize(&solution);

        #[cfg(feature = "timing")]
        println!("Took: {}ms", now.elapsed().as_millis());

        let answer = Answer::new(solution);

        Ok(answer)
    }
}
