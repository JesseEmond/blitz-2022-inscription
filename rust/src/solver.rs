use std::cmp;
use std::error::Error;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use crate::game_interface::{Answer, GameMessage, Question, Totem, TotemAnswer};
use crate::shape_info::{get_rotations, ShapeVariant};

pub struct Solver {}

struct Board {
    size: usize,
    grid: Vec<Vec<bool>>,
    touchpoints: Vec<Vec<i32>>,
    totems: Vec<TotemAnswer>,
}

impl Board {
    fn new(size: usize) -> Board {
        let grid = vec![vec![false; size]; size];
        let mut touchpoints = vec![vec![0; size]; size];
        // Treat borders as touchpoints
        for x in 0..size {
            touchpoints[0][x] += 1;
            touchpoints[size-1][x] += 1;
        }
        for y in 0..size {
            touchpoints[y][0] += 1;
            touchpoints[y][size-1] += 1;
        }
        touchpoints[0][0] += 1;  // Give (0,0) a boost to ensure we set it.
        // TODO "smallest unset at x"
        return Board { size: size, grid: grid, touchpoints: touchpoints, totems: Vec::new() };
    }

    fn mark(&mut self, shape: &ShapeVariant) {
        for (x, y) in shape.coords.iter() {
            let x = *x as usize;
            let y = *y as usize;
            self.grid[y][x] = true;
            // TODO: update "smallest unset at x"
            if y > 0 {
                self.touchpoints[y-1][x] += 1;
            }
            if y + 1 < self.size {
                self.touchpoints[y+1][x] += 1;
            }
            if x > 0 {
                self.touchpoints[y][x-1] += 1;
            }
            if x + 1 < self.size {
                self.touchpoints[y][x+1] += 1;
            }
        }
        self.totems.push(TotemAnswer { shape: shape.shape, coordinates: shape.coords.to_vec() });
    }

    fn fits(&self, shape: &ShapeVariant) -> Option<bool> {
        for (x, y) in shape.coords.iter() {
            if *x < 0 || *x >= self.size as i32 || *y < 0 || *y >= self.size as i32 {
                return None;
            }
            let x = *x as usize;
            let y = *y as usize;
            if self.grid[y][x] {
                return Some(false);
            }
        }
        return Some(true);
    }

    fn num_touchpoints(&self, shape: &ShapeVariant) -> i32 {
        let mut total = 0;
        for (x, y) in shape.coords.iter() {
            let x = *x as usize;
            let y = *y as usize;
            total += self.touchpoints[y][x];
        }
        return total;
    }

    fn move_first_fit_above(&self, shape: &mut ShapeVariant) -> bool {
        loop {
            match self.fits(shape) {
                Some(fitted) => {
                    if fitted {
                        return true
                    } else {
                        for (_x, y) in &mut shape.coords {
                            *y += 1;
                        }
                    }
                },
                None => return false,
            }
        }
    }
}

type ShapeDist = HashMap<Totem, i32>;

fn try_fit(board: &mut Board, mut dist: ShapeDist) -> Option<Vec<TotemAnswer>> {
    loop {
        let mut best_shape: Option<ShapeVariant> = None;
        let mut best_touchpoints: i32 = 0;
        let mut shapes_left = 0;
        for (&totem, &n_totem) in &dist {
            shapes_left += n_totem;
            if n_totem > 0 {
                for variant in get_rotations(totem) {
                    for dx in 0..(board.size as i32 + 1 - variant.width as i32) {
                        let mut variant = variant.clone();
                        for (var_x, _var_y) in &mut variant.coords {
                            *var_x += dx;   
                        }
                        if board.move_first_fit_above(&mut variant) {
                            let touchpoints = board.num_touchpoints(&variant);
                            if touchpoints > best_touchpoints {
                                best_touchpoints = touchpoints;
                                best_shape = Some(variant.clone());
                            }
                        }
                    }
                }
            }
        }
        if shapes_left == 0 {
            return Some(board.totems.clone());
        }
        match &best_shape {
            Some(shape) => {
                board.mark(shape);
                *dist.get_mut(&shape.shape).unwrap() -= 1;
            },
            None => return None
        }
    }
}

fn solve(question: &Question) -> Vec<TotemAnswer> {
    let dist = get_shape_distribution(question);
    let n_squares = question.totems.len() * 4;
    let mut side = (n_squares as f64).sqrt().ceil() as i32;
    loop {
        println!("Trying {0}x{0}...", side);
        let fit = try_fit(&mut Board::new(side as usize), dist.clone());
        if fit.is_some() {
            return fit.unwrap();
        }
        side += 1;
    }
}

fn visualize(answer: &Vec<TotemAnswer>) {
    let mut max_x = -1;
    let mut max_y = -1;
    for totem in answer.iter() {
        for (x, y) in totem.coordinates.iter() {
            max_x = cmp::max(max_x, *x);
            max_y = cmp::max(max_y, *y);
        }
    }
    let w = max_x + 1;
    let h = max_y + 1;
    let mut lines = vec![vec!['.'; w as usize]; h as usize];
    for totem in answer.iter() {
        for (x, y) in totem.coordinates.iter() {
            lines[*y as usize][*x as usize] = format!("{:?}", totem.shape).chars().next().unwrap();
        }
    }
    for line in lines.iter().rev() {
        for c in line.iter() {
            print!("{}", c);
        }
        println!();
    }
}

fn get_shape_distribution(question: &Question) -> ShapeDist {
    let mut map = HashMap::new();
    for totem in &question.totems {
        let count = map.entry(totem.shape).or_insert(0);
        *count += 1;
    }
    return map;
}

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

        
        let now = Instant::now();
        let solution = solve(question);
        println!("Took: {}ms", now.elapsed().as_millis());
        //visualize(&solution);

        let answer = Answer::new(solution);

        Ok(answer)
    }
}
