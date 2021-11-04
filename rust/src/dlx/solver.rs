use crate::{
    game_interface::{Answer, Question, Totem},
    solver::Solver,
};
use hibitset::BitSet;
use itertools::iproduct;

pub struct DlxSolver {}

impl Solver for DlxSolver {
    fn solve(question: &Question) -> Answer {
        let totems: Vec<_> = question.totems.iter().map(|t| t.shape).collect();

        let optimal_dim = ((totems.len() * 4) as f64).sqrt().ceil() as usize;

        solve_for_size(&totems, optimal_dim, optimal_dim).expect("No solution")
    }
}

fn solve_for_size(totems: &[Totem], width: usize, height: usize) -> Option<Answer> {
    let problem_size = totems.len();

    // Total grid cells
    let total_cells = width * height;
    // Total totem cells
    let total_minos = problem_size * 4;
    // Required universe padding for exact board coverage
    let phantoms = total_cells - total_minos;

    // Offset in bitset for the start of cells
    let cells_offset = problem_size + phantoms;
    // Total bitset row width
    let row_size = cells_offset + total_cells;

    println!("Problem size: {}", problem_size);
    println!("Total minos: {}", total_minos);
    println!("Board size: {}x{} ({} cells)", width, height, total_cells);
    println!("Phantoms needed: {}", phantoms);
    println!("Universe size: {}", row_size);

    #[cfg(feature = "timing")]
    let now = std::time::Instant::now();

    let mut rows = Vec::new();
    for (totem_idx, totem) in totems.iter().enumerate() {
        for (x, y) in iproduct!(0..width, 0..height) {
            'variant: for variant in totem.get_rotations() {
                let mut row = BitSet::with_capacity(row_size as u32);
                assert!(!row.add(totem_idx as u32));
                for point in &variant.coords {
                    let point_x = point.0 + x;
                    if point_x >= width {
                        // GOTO count: 1
                        continue 'variant;
                    }
                    let point_idx = cells_offset + (point.1 + y) * width + point_x;
                    if point_idx >= row_size {
                        // GOTO count: 2
                        continue 'variant;
                    }
                    assert!(!row.add(point_idx as u32));
                }
                rows.push(row);
            }
        }
    }
    rows.extend(iproduct!(0..phantoms, 0..total_cells).map(|(idx, pos)| {
        let mut row = BitSet::with_capacity(row_size as u32);
        row.add((problem_size + idx) as u32);
        row.add((cells_offset + pos) as u32);
        row
    }));

    let mut cols = vec![BitSet::with_capacity(rows.len() as u32); row_size];
    for (idx, row) in rows.iter().enumerate() {
        for col in row {
            cols[col as usize].add(idx as u32);
        }
    }

    #[cfg(feature = "timing")]
    let time_delta = now.elapsed().as_millis();

    println!("Total rows: {}", rows.len());

    #[cfg(feature = "timing")]
    println!("Time taken: {}ms", time_delta);

    None
}
