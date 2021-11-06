use crate::{
    dlx::DlxMatrix,
    game_interface::{Answer, Question, Totem, TotemAnswer},
    shape_info::ShapeVariant,
    solver::{macros::solver_boilerplate, Solver},
};
use itertools::iproduct;
use std::cmp;

pub struct DlxSolver {}

impl Solver for DlxSolver {
    fn solve(question: &Question) -> Answer {
        solver_boilerplate! {
            let totems: Vec<_> = question.totems.iter().map(|t| t.shape).collect();
            solve(&totems).expect("No solution found")
        }
    }
}

fn solve(totems: &[Totem]) -> Option<Answer> {
    let optimal_dim = ((totems.len() * 4) as f64).sqrt();

    let mut width = cmp::max(optimal_dim.floor() as i32, 2);
    let mut height = cmp::max(optimal_dim.ceil() as i32, 4);
    for _ in 0..5 {
        if let Some(s) = solve_for_size(&totems, width, height) {
            return Some(s);
        }
        if width < height {
            width += 1;
        } else {
            height += 1;
        }
    }
    None
}

fn solve_for_size(totems: &[Totem], width: i32, height: i32) -> Option<Answer> {
    #[cfg(feature = "timing")]
    let now = std::time::Instant::now();

    let problem_size = totems.len() as i32;

    // Total grid cells
    let total_cells = width * height;
    // Total totem cells
    let total_minos = problem_size * 4;

    // Origin cover requirement
    let origin_cover_idx = problem_size;

    // Offset for the cells columns
    let cells_offset = origin_cover_idx + 1;

    // Total bitset row width
    let row_size = cells_offset + total_cells;

    println!("Problem size: {}", problem_size);
    println!("Total minos: {}", total_minos);
    println!("Board size: {}x{} ({} cells)", width, height, total_cells);
    println!("Universe size: {}", row_size);

    let mut row_idx = 0;
    let mut matrix = DlxMatrix::new();
    let mut mappings: Vec<(&'static ShapeVariant, (i32, i32))> = Vec::new();
    for (totem_idx, totem) in totems.iter().enumerate() {
        for shape in totem.get_rotations() {
            for (y, x) in iproduct!(0..height, 0..width) {
                if x + (shape.width as i32) <= width && y + (shape.height as i32) <= height {
                    #[cfg(feature = "visualize")]
                    let mut show = vec![b' '; row_size as usize];

                    matrix.set(row_idx, totem_idx as i32);
                    #[cfg(feature = "visualize")]
                    {
                        show[totem_idx] = b'+';
                    }

                    for point in &shape.coords {
                        let point_x = x + point.0 as i32;
                        let point_y = y + point.1 as i32;
                        if point_x == 0 && point_y == 0 {
                            matrix.set(row_idx, origin_cover_idx);
                            #[cfg(feature = "visualize")]
                            {
                                show[origin_cover_idx as usize] = b'+';
                            }
                        }
                        let point_idx = cells_offset + point_y * width + point_x;
                        matrix.set(row_idx, point_idx);
                        #[cfg(feature = "visualize")]
                        {
                            show[point_idx as usize] = b'+';
                        }
                    }

                    mappings.insert(row_idx as usize, (shape, (x, y)));

                    #[cfg(feature = "visualize")]
                    println!(
                        "{:4} ({:2}, {:2}) {:?} {}",
                        row_idx,
                        x,
                        y,
                        shape.shape,
                        String::from_utf8_lossy(&show),
                    );

                    row_idx += 1;
                }
            }
        }
    }
    for col in cells_offset..row_size {
        matrix.mark_optional(col);
    }

    println!("Total rows: {}", matrix.rows());

    #[cfg(feature = "timing")]
    println!("Time taken: {}ms", now.elapsed().as_millis());

    #[cfg(feature = "timing")]
    let now = std::time::Instant::now();

    let solution = matrix.solve_first();

    #[cfg(feature = "timing")]
    println!("Time taken: {}ms", now.elapsed().as_millis());

    println!("Solution: {:?}", solution);

    if let Some(cols) = solution {
        let answer = cols
            .into_iter()
            .filter_map(|idx| {
                mappings
                    .get(idx as usize)
                    .map(|&(variant, (offset_x, offset_y))| {
                        TotemAnswer::new(
                            variant.shape,
                            variant
                                .coords
                                .iter()
                                .map(|(x, y)| (offset_x as usize + x, offset_y as usize + y))
                                .collect(),
                        )
                    })
            })
            .collect();
        Some(Answer::new(answer))
    } else {
        None
    }
}
