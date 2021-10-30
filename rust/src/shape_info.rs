use crate::game_interface::{CoordinatePair, Totem};

#[derive(Clone)]
pub struct ShapeVariant {
    pub shape: Totem,
    pub coords: [CoordinatePair; 4],
    pub width: i8,
    pub height: i8
}

pub fn get_rotations(totem: Totem) -> Vec<ShapeVariant> {
    return match totem {
        Totem::O => vec![
            // OO
            // OO
            ShapeVariant { shape: Totem::O, coords: [(0,0), (0,1), (1,0), (1,1)],
                           width: 2, height: 2 },
        ],
        Totem::I => vec![
            // IIII
            ShapeVariant { shape: Totem::I, coords: [(0,0), (1,0), (2,0), (3,0)],
                           width: 4, height: 1 },
            // I
            // I
            // I
            // I
            ShapeVariant { shape: Totem::I, coords: [(0,0), (0,1), (0,2), (0,3)],
                           width: 1, height: 4 },
        ],
        Totem::J => vec![
            //  J
            //  J
            // JJ
            ShapeVariant { shape: Totem::J, coords: [(0,0), (1,0), (1,1), (1,2)],
                           width: 2, height: 3 },
            // J
            // JJJ
            ShapeVariant { shape: Totem::J, coords: [(0,1), (0,0), (1,0), (2,0)],
                           width: 3, height: 2 },
            // JJ
            // J
            // J
            ShapeVariant { shape: Totem::J, coords: [(0,0), (0,1), (0,2), (1,2)],
                           width: 2, height: 3 },
            // JJJ
            //   J
            ShapeVariant { shape: Totem::J, coords: [(0,1), (1,1), (2,1), (2,0)],
                           width: 3, height: 2 },
        ],
        Totem::L => vec![
            // L
            // L
            // LL
            ShapeVariant { shape: Totem::L, coords: [(0,2), (0,1), (0,0), (1,0)],
                           width: 2, height: 3 },
            //   L
            // LLL
            ShapeVariant { shape: Totem::L, coords: [(0,0), (1,0), (2,0), (2,1)],
                           width: 3, height: 2 },
            // LL
            //  L
            //  L
            ShapeVariant { shape: Totem::L, coords: [(0,2), (1,2), (1,1), (1,0)],
                           width: 2, height: 3 },
            // LLL
            // L
            ShapeVariant { shape: Totem::L, coords: [(0,0), (0,1), (1,1), (2,1)],
                           width: 3, height: 2 },
        ],
        Totem::S => vec![
            // S
            // SS
            //  S
            ShapeVariant { shape: Totem::S, coords: [(0,2), (0,1), (1,1), (1,0)],
                           width: 2, height: 3 },
            //  SS
            // SS
            ShapeVariant { shape: Totem::S, coords: [(0,0), (1,0), (1,1), (2,1)],
                           width: 3, height: 2 },
        ],
        Totem::Z => vec![
            //  Z
            // ZZ
            // Z
            ShapeVariant { shape: Totem::Z, coords: [(0,0), (0,1), (1,1), (1,2)],
                           width: 2, height: 3 },
            // ZZ
            //  ZZ
            ShapeVariant { shape: Totem::Z, coords: [(0,1), (1,1), (1,0), (2,0)],
                           width: 3, height: 2 },
        ],
        Totem::T => vec![
            // TTT
            //  T
            ShapeVariant { shape: Totem::T, coords: [(0,1), (1,1), (2,1), (1,0)],
                           width: 3, height: 2 },
            // T
            // TT
            // T
            ShapeVariant { shape: Totem::T, coords: [(0,2), (0,1), (1,1), (0,0)],
                           width: 2, height: 3 },
            //  T
            // TTT
            ShapeVariant { shape: Totem::T, coords: [(0,0), (1,0), (2,0), (1,1)],
                           width: 3, height: 2 },
            //  T
            // TT
            //  T
            ShapeVariant { shape: Totem::T, coords: [(1,2), (1,1), (1,0), (0,1)],
                           width: 2, height: 3 },
        ],
    }
}