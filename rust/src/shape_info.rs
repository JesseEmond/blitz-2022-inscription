// Lookup information about each totem, including its rotations, width, height.
// Also includes u64 'masks' used to quickly check for collisions.
// Note that the masks are inverted vertically to match how they're checked, so they are
// visually confusing.

use crate::game_interface::{Point, Totem, TOTEM_COUNT};

const I_VARIANTS: [ShapeVariant; 2] = [
    // IIII
    ShapeVariant {
        shape: Totem::I,
        coords: [(0, 0), (1, 0), (2, 0), (3, 0)],
        width: 4,
        height: 1,
        masks: [
            0b1111u64 << 60,
            0, 0, 0
        ]
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
        masks: [
            0b1u64 << 63,
            0b1u64 << 63,
            0b1u64 << 63,
            0b1u64 << 63,
        ]
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
        masks: [
            0b11u64 << 62,
            0b01u64 << 62,
            0b01u64 << 62,
            0
        ]
    },
    // J
    // JJJ
    ShapeVariant {
        shape: Totem::J,
        coords: [(0, 1), (0, 0), (1, 0), (2, 0)],
        width: 3,
        height: 2,
        masks: [
            0b111u64 << 61,
            0b100u64 << 61,
            0, 0
        ]
    },
    // JJ
    // J
    // J
    ShapeVariant {
        shape: Totem::J,
        coords: [(0, 0), (0, 1), (0, 2), (1, 2)],
        width: 2,
        height: 3,
        masks: [
            0b10u64 << 62,
            0b10u64 << 62,
            0b11u64 << 62,
            0
        ]
    },
    // JJJ
    //   J
    ShapeVariant {
        shape: Totem::J,
        coords: [(0, 1), (1, 1), (2, 1), (2, 0)],
        width: 3,
        height: 2,
        masks: [
            0b001u64 << 61,
            0b111u64 << 61,
            0, 0
        ]
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
        masks: [
            0b11u64 << 62,
            0b10u64 << 62,
            0b10u64 << 62,
            0
        ]
    },
    //   L
    // LLL
    ShapeVariant {
        shape: Totem::L,
        coords: [(0, 0), (1, 0), (2, 0), (2, 1)],
        width: 3,
        height: 2,
        masks: [
            0b111u64 << 61,
            0b001u64 << 61,
            0, 0
        ]
    },
    // LL
    //  L
    //  L
    ShapeVariant {
        shape: Totem::L,
        coords: [(0, 2), (1, 2), (1, 1), (1, 0)],
        width: 2,
        height: 3,
        masks: [
            0b01u64 << 62,
            0b01u64 << 62,
            0b11u64 << 62,
            0
        ]
    },
    // LLL
    // L
    ShapeVariant {
        shape: Totem::L,
        coords: [(0, 0), (0, 1), (1, 1), (2, 1)],
        width: 3,
        height: 2,
        masks: [
            0b100u64 << 61,
            0b111u64 << 61,
            0, 0
        ]
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
        masks: [
            0b11u64 << 62,
            0b11u64 << 62,
            0, 0
        ]
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
        masks: [
            0b01u64 << 62,
            0b11u64 << 62,
            0b10u64 << 62,
            0
        ]
    },
    //  SS
    // SS
    ShapeVariant {
        shape: Totem::S,
        coords: [(0, 0), (1, 0), (1, 1), (2, 1)],
        width: 3,
        height: 2,
        masks: [
            0b110u64 << 61,
            0b011u64 << 61,
            0, 0
        ]
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
        masks: [
            0b010u64 << 61,
            0b111u64 << 61,
            0, 0
        ]
    },
    // T
    // TT
    // T
    ShapeVariant {
        shape: Totem::T,
        coords: [(0, 2), (0, 1), (1, 1), (0, 0)],
        width: 2,
        height: 3,
        masks: [
            0b10u64 << 62,
            0b11u64 << 62,
            0b10u64 << 62,
            0
        ]
    },
    //  T
    // TTT
    ShapeVariant {
        shape: Totem::T,
        coords: [(0, 0), (1, 0), (2, 0), (1, 1)],
        width: 3,
        height: 2,
        masks: [
            0b111u64 << 61,
            0b010u64 << 61,
            0, 0
        ]
    },
    //  T
    // TT
    //  T
    ShapeVariant {
        shape: Totem::T,
        coords: [(1, 2), (1, 1), (1, 0), (0, 1)],
        width: 2,
        height: 3,
        masks: [
            0b01u64 << 62,
            0b11u64 << 62,
            0b01u64 << 62,
            0
        ]
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
        masks: [
            0b10u64 << 62,
            0b11u64 << 62,
            0b01u64 << 62,
            0
        ]
    },
    // ZZ
    //  ZZ
    ShapeVariant {
        shape: Totem::Z,
        coords: [(0, 1), (1, 1), (1, 0), (2, 0)],
        width: 3,
        height: 2,
        masks: [
            0b011u64 << 61,
            0b110u64 << 61,
            0, 0
        ]
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
    // Note that those visually don't match up, since positive y is up.
    masks: [u64; 4],
}

impl ShapeVariant {
    pub fn get_rotations(totem: &Totem) -> &'static [ShapeVariant] {
        VARIANTS[*totem as usize]
    }

    // Minimum (w, h) needed for this totem (based on its rotation with smallest width).
    pub fn minimum_dims(totem: &Totem) -> (usize, usize) {
        let mut dims = (5, 5);  // all shapes are smaller than this
        for rotation in ShapeVariant::get_rotations(totem) {
            if (rotation.width as usize) < dims.0 {
                dims = (rotation.width as usize, rotation.height as usize);
            }
        }
        dims
    }

    pub fn offset_by(&self, x: usize, y: usize) -> ShapeVariant {
        let mut coords = self.coords.clone();
        for (dx, dy) in &mut coords {
            *dx += x;
            *dy += y;
        }
        ShapeVariant {
            shape: self.shape,
            coords: coords,
            width: self.width,
            height: self.height,
            masks: self.masks,  // NOTE: this makes the masks no longer valid horizontally. Not used.
        }
    }

     // Return the mask at a given local shape y position.
    pub fn mask_at(&self, global_x: usize, local_y: usize) -> u64 {
        let mask = unsafe { self.masks.get_unchecked(local_y) };
        mask >> global_x
    }
}
