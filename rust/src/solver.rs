use crate::{
    game_interface::{Answer, GameMessage, Question, Totem, TotemAnswer},
    shape_info::ShapeVariant,
};
use std::{error::Error, cmp, time::Instant};

struct Board {
    size: usize,
    grid: Vec<Vec<bool>>,
    touchpoints: Vec<Vec<u32>>,
    totems: Vec<TotemAnswer>,
}

impl Board {
    fn new(size: usize, answer_size: usize) -> Board {
        let grid = vec![vec![false; size]; size];
        let mut touchpoints = vec![vec![0; size]; size];
        // Treat borders as touchpoints
        for i in 0..size {
            touchpoints[0][i] += 1;
            touchpoints[size - 1][i] += 1;
            touchpoints[i][0] += 1;
            touchpoints[i][size - 1] += 1;
        }
        touchpoints[0][0] += 1; // Give (0,0) a boost to ensure we set it.
                                // TODO "smallest unset at x"
        Board {
            size,
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
            if *y + 1 < self.size {
                self.touchpoints[*y + 1][*x] += 1;
            }
            if *x > 0 {
                self.touchpoints[*y][*x - 1] += 1;
            }
            if *x + 1 < self.size {
                self.touchpoints[*y][*x + 1] += 1;
            }
        }
        self.totems
            .push(TotemAnswer::new(shape.shape, shape.coords.to_vec()));
    }

    fn fits(&self, shape: &ShapeVariant) -> Option<bool> {
        for (x, y) in &shape.coords {
            if *x >= self.size || *y >= self.size {
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

fn try_fit(board: &mut Board, mut dist: ShapeDist) -> Option<Vec<TotemAnswer>> {
    loop {
        let mut best_shape: Option<ShapeVariant> = None;
        let mut best_touchpoints: u32 = 0;
        let mut shapes_left = 0;
        for totem in Totem::iter() {
            let n_totem = dist[*totem as usize];
            shapes_left += n_totem;
            if n_totem > 0 {
                for variant in ShapeVariant::get_rotations(totem) {
                    for dx in 0..(board.size + 1 - variant.width) {
                        let mut variant = variant.clone();
                        for point in &mut variant.coords {
                            point.0 += dx;
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

fn solve(question: &Question) -> Vec<TotemAnswer> {
    let dist = get_shape_distribution(question);
    let answer_size = question.totems.len();
    let n_squares = answer_size * 4;
    let mut side = cmp::max((n_squares as f64).sqrt().ceil() as usize, 4);
    loop {
        println!("Trying {0}x{0}...", side);
        if let Some(fit) = try_fit(&mut Board::new(side, answer_size), dist) {
            return fit;
        }
        side += 1;
    }
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
}

fn get_shape_distribution(question: &Question) -> ShapeDist {
    let mut dist: ShapeDist = [0; 7];
    for totem in &question.totems {
        dist[totem.shape as usize] += 1;
    }
    dist
}

pub struct Solver {}

impl Solver {
    /// Initialize your solver
    ///
    /// This method should be used to initialize some
    /// variables you will need throughout the challenge.
    pub fn new() -> Self {
        Solver {}
    }

    /// Answer the question
    ///
    /// This is where the magic happens, for now the
    /// answer is a single 'I'. I bet you can do better ;)
    pub fn get_answer(&self, game_message: &GameMessage) -> Result<Answer, Box<dyn Error>> {
        let question = &game_message.payload;
        println!("Received question with {} totems.", question.totems.len());

        #[cfg(feature = "timing")]
        let now = Instant::now();

        let solution = solve(question);

        #[cfg(feature = "timing")]
        println!("Took: {}ms", now.elapsed().as_millis());

        #[cfg(feature = "visualize")]
        visualize(&solution);

        let answer = Answer::new(solution);

        Ok(answer)
    }
}
