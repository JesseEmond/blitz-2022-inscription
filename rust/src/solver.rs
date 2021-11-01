use crate::game_interface::{Answer, Question};

pub trait Solver {
    fn solve(question: &Question) -> Answer;
}

#[allow(clippy::single_component_path_imports)]
pub mod macros {
    macro_rules! solver_boilerplate {
        ($($tt:tt)*) => {
            #[cfg(feature = "timing")]
            let now = std::time::Instant::now();

            let answer: crate::game_interface::Answer = { $($tt)* };

            #[cfg(feature = "timing")]
            let delta = now.elapsed().as_millis();

            #[cfg(feature = "visualize")]
            crate::solver::visualize(&answer);

            #[cfg(feature = "timing")]
            println!("Took: {}ms", delta);

            answer
        };
    }

    pub(crate) use solver_boilerplate;
}

#[cfg(feature = "visualize")]
pub fn visualize(answer: &Answer) {
    use crate::{game_interface::TOTEM_COUNT, scoring::score};

    static GLYPHS: [u8; TOTEM_COUNT] = [b'I', b'J', b'L', b'O', b'S', b'T', b'Z'];

    let max_x = answer
        .totems
        .iter()
        .flat_map(|t| t.coordinates.iter().map(|p| p.0))
        .max()
        .unwrap();

    let max_y = answer
        .totems
        .iter()
        .flat_map(|t| t.coordinates.iter().map(|p| p.1))
        .max()
        .unwrap();

    let w = max_x + 1;
    let h = max_y + 1;

    let mut lines = vec![vec![b'.'; w]; h];
    for totem in &answer.totems {
        for (x, y) in &totem.coordinates {
            lines[*y][*x] = GLYPHS[totem.shape as usize];
        }
    }

    for line in lines.into_iter().rev() {
        println!("{}", String::from_utf8(line).unwrap());
    }
    println!("{}x{}, score={}", w, h, score(answer.totems.len(), w, h));
}
