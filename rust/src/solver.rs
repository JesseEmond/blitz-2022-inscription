use crate::{
    exact_solver,
    game_interface::{Answer, GameMessage, Question, Totem, TotemAnswer, TotemQuestion},
    scoring::{score, Dims, OptimalDimensions},
    shape_info::{ShapeDist, ShapeVariant},
};
use rand::{
    self,
    distributions::{Distribution, Uniform},
    seq::SliceRandom,
};
use std::{error::Error, cmp, thread, time::Instant};

struct Board {
    width: usize,
    height: usize,
    // TODO: consider making this 2 columns per row (half the "fit" checks, since the max perfect dim is 32x32 for 256 totems)
    // i.e. 2x u32s in a single u64, with the shape masks precomputed like that.
    masked_grid: Vec<u64>,
    touchpoints: Vec<Vec<u32>>,
    totems: Vec<TotemAnswer>,
    first_unset_y_at_x: Vec<usize>,
}

impl Board {
    fn new(width: usize, height: usize, answer_size: usize) -> Board {
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
        touchpoints[0][0] += 100; // Give (0,0) a boost to ensure we set it.
        Board {
            width,
            height,
            masked_grid: vec![0; height + 3], // padding since we check zeroed-out masks past the height, for speed.
            touchpoints,
            totems: Vec::with_capacity(answer_size),
            first_unset_y_at_x: vec![0; width],
        }
    }

    fn mark(&mut self, shape: &ShapeVariant, left_x: usize, bottom_y: usize) {
        for (x, y) in &shape.coords {
            let x = *x + left_x;
            let y = *y + bottom_y;
            if y > 0 {
                self.touchpoints[y - 1][x] += 1;
            }
            if y + 1 < self.height {
                self.touchpoints[y + 1][x] += 1;
            }
            if x > 0 {
                self.touchpoints[y][x - 1] += 1;
            }
            if x + 1 < self.width {
                self.touchpoints[y][x + 1] += 1;
            }
            if self.first_unset_y_at_x[x] == y {
                self.first_unset_y_at_x[x] += 1;
            }
        }
        for dy in 0..shape.height {
            let y = bottom_y + dy as usize;
            let mask = shape.mask_at(left_x, dy as usize);
            self.masked_grid[y] |= mask;
        }
        let shape = shape.offset_by(left_x, bottom_y);
        self.totems
            .push(TotemAnswer::new(shape.shape, shape.coords.to_vec()));
    }

    fn fits(&self, shape: &ShapeVariant, left_x: usize, bottom_y: usize) -> bool {
        let mut fit = 0;
        for dy in 0..4 {
            let y = bottom_y + dy as usize;
            let shape_mask = shape.mask_at(left_x, dy as usize);
            let board_mask = unsafe { self.masked_grid.get_unchecked(y) };
            fit |= shape_mask & board_mask;
        }
        return fit == 0;
    }

    fn num_touchpoints(&self, shape: &ShapeVariant, left_x: usize, bottom_y: usize) -> u32 {
        let mut total = 0;
        for (x, y) in &shape.coords {
            let x = *x + left_x;
            let y = *y + bottom_y;
            total += self.touchpoints[y][x];
        }
        total
    }

    fn min_y_for_shape(&self, shape: &ShapeVariant, left_x: usize) -> usize {
        let mut min_y = 0;
        for (dx, dy) in &shape.coords {
            let x = left_x + *dx;
            if self.first_unset_y_at_x[x] > min_y + *dy {
                min_y = self.first_unset_y_at_x[x] - *dy;
            }
        }
        min_y
    }

    fn move_first_fit_above(&self, shape: &ShapeVariant, left_x: usize, out_y: &mut usize) -> bool {
        let min_y = self.min_y_for_shape(shape, left_x);
        for y in min_y..(self.height + 1 - shape.height as usize) {
            if self.fits(shape, left_x, y) {
                *out_y = y;
                return true;
            }
        }
        false
    }
}

struct Placement {
    totem: Totem,
    rotation_index: usize,
    x: usize,
    y: usize,
}

fn try_gravity_greedy_fit(board: &mut Board, mut dist: ShapeDist) -> Option<Vec<TotemAnswer>> {
    let mut rng = rand::thread_rng();
    let mut options = Vec::with_capacity(7 * 4 * board.width);  // 7 shapes, 4 rotations max, 'width' x positions.
    loop {
        options.clear();
        let mut shapes_left = 0;
        let mut max_touchpoints = 0;

        for totem in Totem::iter() {
            let n_totem = dist[*totem as usize];
            shapes_left += n_totem;
            if n_totem > 0 {
                for (rot_idx, variant) in ShapeVariant::get_rotations(totem).iter().enumerate() {
                    for dx in 0..(board.width as i32 + 1 - variant.width as i32) {
                        let x = dx as usize;
                        let mut y = 0;
                        if board.move_first_fit_above(&variant, x, &mut y) {
                            let touchpoints = board.num_touchpoints(&variant, x, y);
                            if touchpoints > max_touchpoints {
                                max_touchpoints = touchpoints;
                                options.clear();
                            }
                            if touchpoints == max_touchpoints {
                                options.push(Placement {
                                    totem: variant.shape, rotation_index: rot_idx,
                                    x: x, y: y
                                    });
                            }
                        }
                    }
                }
            }
        }
        if shapes_left == 0 {
            return Some(board.totems.clone());
        }
        if options.is_empty() {
            return None;
        }
        let placement = options.choose(&mut rng).unwrap();
        let rotations = ShapeVariant::get_rotations(&placement.totem);
        let shape = rotations.iter().nth(placement.rotation_index).unwrap();
        board.mark(&shape, placement.x, placement.y);
        dist[shape.shape as usize] -= 1;
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

fn try_greedy_solve(width: usize, height: usize, num_totems: usize, dist: ShapeDist) -> Option<Vec<TotemAnswer>> {
    let mut attempts = 1000;
    if num_totems >= 256 {  // Takes too long in this case.
        attempts = 100;
    }
    for _ in 0..attempts {
        if let Some(sln) = try_gravity_greedy_fit(&mut Board::new(width, height, num_totems), dist) {
            return Some(sln);
        }
    }
    None
}

fn do_solve(width: usize, height: usize, num_totems: usize, dist: &ShapeDist,
            greedy: bool, greedy_multithreaded: bool) -> Option<Vec<TotemAnswer>> {
    if greedy {
        if greedy_multithreaded {
            let mut handles = vec![];
            // From tests, we think we're on a c5a.2xlarge, so 4 cores, 8 hyperthreaded.
            // As IIUC going up to 8 would hurt, since we're doing purely CPU processing and not much IO:
            // https://www.credera.com/insights/whats-in-a-vcpu-state-of-amazon-ec2-in-2018
            for _ in 0..3 {
                let dist = *dist;
                handles.push(thread::spawn(move || {
                    try_greedy_solve(width, height, num_totems, dist)
                }));
            }
            for handle in handles {
                if let Some(sln) = handle.join().unwrap() {
                    return Some(sln);
                }
            }
            None
        } else {
            try_greedy_solve(width, height, num_totems, *dist)
        }
    } else {
        exact_solver::solve(width, height, *dist)
    }
}

fn solve(question: &Question, level: usize,
         optimal_dims: &OptimalDimensions, greedy: bool,
         greedy_multithreaded: bool) -> Vec<TotemAnswer> {
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
        print!("Trying {}x{}... would give {}... ", *w, *h, score(answer_size, *w, *h));
        if let Some(fit) = do_solve(*w, *h, answer_size, &dist, greedy, greedy_multithreaded) {
            println!("OK!");
            return fit;
        } else if *w != *h {
            if let Some(fit) = do_solve(*h, *w, answer_size, &dist, greedy, greedy_multithreaded) {
                // Because of our (0, 0) constraint, sometimes the rotation works.
                // We run fast enough to just try both.
                println!("OK!  (with rotation {}x{})", *h, *w);
                return fit;
            }
        }
        println!("No fit found.");
    }
    println!("!!! FAILED TO FIND SOLUTION. Should increase ranges in 'optimal dims'.");
    // If we can't find a solution with a 4*totemx4*totem grid..... we deserve to crash
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
    println!("{}x{}, {} totems, score={}", w, h, answer.len(), score(answer.len(), w, h));
}

fn get_shape_distribution(question: &Question) -> ShapeDist {
    let mut dist: ShapeDist = [0; 7];
    for totem in &question.totems {
        dist[totem.shape as usize] += 1;
    }
    dist
}

// Returns 95% confidence interval for the success probability given a given amount of 'successes'
// over a given amount of 'trials'.
// https://en.wikipedia.org/wiki/Binomial_proportion_confidence_interval
// Using the Wilson score interval.
fn binomial_confidence_interval(successes: u64, trials: u64) -> (f64, f64) {
    let n = trials as f64;
    let p_hat = successes as f64 / n;
    let z: f64 = 1.96;  // alpha = 1 - 0.95 = 0.05 for 95% confidence, 1-alpha/2=0.975 => z=1.96
    // Give names to variables for the general form of a +- b * c.sqrt(), to make it (a bit) easier to follow.
    let a = (p_hat + z * z / (2f64 * n)) / (1f64 + z * z / n);
    let b = z / (1f64 + z * z / n);
    let c = p_hat * (1f64 - p_hat) / n + z * z / (4f64 * n * n);
    (a - b * c.sqrt(), a + b * c.sqrt())
}

#[allow(dead_code)]
fn debug_full_packing_probability(level: usize, greedy: bool, greedy_multithreaded: bool, 
                                  optimal_dims: &OptimalDimensions) {
    let num_totems = 1 << level;
    let mut rng = rand::thread_rng();
    let mut total_runs = 0;
    let mut perfect_packs = 0;
    let mut last_time: std::time::Instant = std::time::Instant::now();
    let start_time = std::time::Instant::now();
    loop {
        let mut questions: Vec<TotemQuestion> = Vec::with_capacity(num_totems);
        let die = Uniform::from(0..7);
        for _ in 0..num_totems {
            let idx: usize = die.sample(&mut rng);
            let shape: Totem = unsafe { std::mem::transmute(idx) };
            questions.push(TotemQuestion { shape });
        }
        let dist = get_shape_distribution(&Question { totems: questions });
        let (w, h) = optimal_dims.level_dims(level).next().unwrap();
        if let Some(_sln) = do_solve(*w, *h, num_totems, &dist, greedy, greedy_multithreaded) {
            perfect_packs += 1;
            // visualize(&_sln);  // To visually make sure the solutions are valid.
        }
        total_runs += 1;
        if last_time.elapsed().as_secs_f64() > 0.5 {
            let total_time = start_time.elapsed().as_secs_f64();
            let pack_speed = (total_runs as f64) / total_time;
            let pack_ratio = (perfect_packs as f32) / (total_runs as f32);
            let (lower_bound, upper_bound) = binomial_confidence_interval(perfect_packs, total_runs);
            println!("{} / {} perfect packs (p={:.1}%   alpha=0.05 interval=[{:.1}%, {:.1}%]),   ~{:.2}s/it",
                     perfect_packs, total_runs, pack_ratio * 100f32, lower_bound * 100f64, upper_bound * 100f64,
                     1f64 / pack_speed);
            last_time = std::time::Instant::now();
        }
    }
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
        let greedy = num_totems > 8;  // TODO: decide on breakpoint
        println!("Received question with {} totems.", num_totems);

        let inferred_level = (num_totems as f64).log2().ceil() as usize;

        // If you're trying to estimate the probability of hitting a perfect fit, uncomment the following.
        // You can set multithreading to off if you want to profile the implementation with less noise.
        //debug_full_packing_probability(inferred_level, greedy, /*greedy_multithreaded=*/true, &self.optimal_dims);

        let (optimal_w, optimal_h) = self.optimal_dims.level_dims(inferred_level).next().unwrap();
        println!("Optimal dims for level {} would be {}x{}, which would give score {}",
                 inferred_level + 1, optimal_w, optimal_h, score(num_totems, *optimal_w, *optimal_h));

        #[cfg(feature = "timing")]
        let now = Instant::now();

        let solution = solve(question, inferred_level, &self.optimal_dims, greedy, /*greedy_multithreaded=*/true);

        // TODO quick visual indication of whether we got the optimal score for level

        #[cfg(feature = "visualize")]
        visualize(&solution);

        #[cfg(feature = "timing")]
        println!("Took: {}ms", now.elapsed().as_millis());

        let answer = Answer::new(solution);

        Ok(answer)
    }
}
