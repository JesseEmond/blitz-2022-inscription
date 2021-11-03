use crate::game_interface::{Point, TOTEM_COUNT, Totem};

const I_VARIANTS: [ShapeVariant; 2] = [
    // IIII
    ShapeVariant {
        shape: Totem::I,
        coords: [(0, 0), (1, 0), (2, 0), (3, 0)],
        width: 4,
        height: 1,
    },
    // I
    // I
    // I
    // I
    ShapeVariant {
        shape: Totem::I,
        coords: [(0, 0), (0, 1), (0, 2), (0, 3)],
        width: 1,
        height: 4,
    },
];
const J_VARIANTS: [ShapeVariant; 4] = [
    //  J
    //  J
    // JJ
    ShapeVariant {
        shape: Totem::J,
        coords: [(0, 0), (1, 0), (1, 1), (1, 2)],
        width: 2,
        height: 3,
    },
    // J
    // JJJ
    ShapeVariant {
        shape: Totem::J,
        coords: [(0, 1), (0, 0), (1, 0), (2, 0)],
        width: 3,
        height: 2,
    },
    // JJ
    // J
    // J
    ShapeVariant {
        shape: Totem::J,
        coords: [(0, 0), (0, 1), (0, 2), (1, 2)],
        width: 2,
        height: 3,
    },
    // JJJ
    //   J
    ShapeVariant {
        shape: Totem::J,
        coords: [(0, 1), (1, 1), (2, 1), (2, 0)],
        width: 3,
        height: 2,
    },
];
const L_VARIANTS: [ShapeVariant; 4] = [
    // L
    // L
    // LL
    ShapeVariant {
        shape: Totem::L,
        coords: [(0, 2), (0, 1), (0, 0), (1, 0)],
        width: 2,
        height: 3,
    },
    //   L
    // LLL
    ShapeVariant {
        shape: Totem::L,
        coords: [(0, 0), (1, 0), (2, 0), (2, 1)],
        width: 3,
        height: 2,
    },
    // LL
    //  L
    //  L
    ShapeVariant {
        shape: Totem::L,
        coords: [(0, 2), (1, 2), (1, 1), (1, 0)],
        width: 2,
        height: 3,
    },
    // LLL
    // L
    ShapeVariant {
        shape: Totem::L,
        coords: [(0, 0), (0, 1), (1, 1), (2, 1)],
        width: 3,
        height: 2,
    },
];
const O_VARIANTS: [ShapeVariant; 1] = [
    // OO
    // OO
    ShapeVariant {
        shape: Totem::O,
        coords: [(0, 0), (0, 1), (1, 0), (1, 1)],
        width: 2,
        height: 2,
    },
];
const S_VARIANTS: [ShapeVariant; 2] = [
    // S
    // SS
    //  S
    ShapeVariant {
        shape: Totem::S,
        coords: [(0, 2), (0, 1), (1, 1), (1, 0)],
        width: 2,
        height: 3,
    },
    //  SS
    // SS
    ShapeVariant {
        shape: Totem::S,
        coords: [(0, 0), (1, 0), (1, 1), (2, 1)],
        width: 3,
        height: 2,
    },
];
const T_VARIANTS: [ShapeVariant; 4] = [
    // TTT
    //  T
    ShapeVariant {
        shape: Totem::T,
        coords: [(0, 1), (1, 1), (2, 1), (1, 0)],
        width: 3,
        height: 2,
    },
    // T
    // TT
    // T
    ShapeVariant {
        shape: Totem::T,
        coords: [(0, 2), (0, 1), (1, 1), (0, 0)],
        width: 2,
        height: 3,
    },
    //  T
    // TTT
    ShapeVariant {
        shape: Totem::T,
        coords: [(0, 0), (1, 0), (2, 0), (1, 1)],
        width: 3,
        height: 2,
    },
    //  T
    // TT
    //  T
    ShapeVariant {
        shape: Totem::T,
        coords: [(1, 2), (1, 1), (1, 0), (0, 1)],
        width: 2,
        height: 3,
    },
];
const Z_VARIANTS: [ShapeVariant; 2] = [
    //  Z
    // ZZ
    // Z
    ShapeVariant {
        shape: Totem::Z,
        coords: [(0, 0), (0, 1), (1, 1), (1, 2)],
        width: 2,
        height: 3,
    },
    // ZZ
    //  ZZ
    ShapeVariant {
        shape: Totem::Z,
        coords: [(0, 1), (1, 1), (1, 0), (2, 0)],
        width: 3,
        height: 2,
    },
];

const VARIANTS: [&[ShapeVariant]; TOTEM_COUNT] = [
    &I_VARIANTS,
    &J_VARIANTS,
    &L_VARIANTS,
    &O_VARIANTS,
    &S_VARIANTS,
    &T_VARIANTS,
    &Z_VARIANTS,
];

#[derive(Clone, Debug)]
pub struct ShapeVariant {
    pub shape: Totem,
    pub coords: [Point; 4],
    pub width: usize,
    pub height: usize,
}

impl ShapeVariant {
    pub fn get_rotations(totem: &Totem) -> &'static [ShapeVariant] {
        VARIANTS[*totem as usize]
    }
}
