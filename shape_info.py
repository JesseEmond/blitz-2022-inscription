from typing import List, Tuple

from game_interface import CoordinatePair, Totem


def parse_shape(*lines) -> List[CoordinatePair]:
    lines = lines[
        ::-1
    ]  # Inverse y-axis so that (0, 0) is bottom-left instead of top-left.
    return [
        (x, y)  # y axis going
        for y in range(len(lines))
        for x in range(len(lines[y]))
        if lines[y][x] != " "
    ]


SHAPE_VARIANTS = {
    "L": [
        parse_shape("L ", "L ", "LL"),
        parse_shape("LLL", "L "),
        parse_shape("LL", " L", " L"),
        parse_shape("  L", "LLL"),
    ],
    "J": [
        parse_shape(" J", " J", "JJ"),
        parse_shape("J  ", "JJJ"),
        parse_shape("JJ", "J ", "J "),
        parse_shape("JJJ", "  J"),
    ],
    "I": [
        parse_shape("IIII"),
        parse_shape("I", "I", "I", "I"),
    ],
    "T": [
        parse_shape(" T ", "TTT"),
        parse_shape("T ", "TT", "T "),
        parse_shape("TTT", " T "),
        parse_shape(" T", "TT", " T"),
    ],
    "S": [
        parse_shape("S", "SS", " S"),
        parse_shape(" SS", "SS "),
    ],
    "Z": [
        parse_shape(
            "ZZ ",
            " ZZ",
        ),
        parse_shape(" Z", "ZZ", "Z "),
    ],
    "O": [
        parse_shape("OO", "OO"),
    ],
}

SHAPE_VARIANT_DIMS = {
    shape: [
        (max(x for x, _ in variant) + 1, max(y for _, y in variant) + 1)
        for variant in SHAPE_VARIANTS[shape]
    ]
    for shape in SHAPE_VARIANTS
}


def variants(shape: Totem) -> List[List[CoordinatePair]]:
    return SHAPE_VARIANTS[shape]


def variants_with_dims(
    shape: Totem,
) -> List[Tuple[List[CoordinatePair], CoordinatePair]]:
    return [
        (SHAPE_VARIANTS[shape][i], SHAPE_VARIANT_DIMS[shape][i])
        for i in range(len(SHAPE_VARIANTS[shape]))
    ]
