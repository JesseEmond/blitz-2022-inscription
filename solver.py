import time
from typing import List, Tuple

from game_interface import Answer, CoordinatePair, GameMessage, Totem, TotemAnswer


def parse_shape(*lines) -> List[CoordinatePair]:
    lines = lines[::-1]  # Inverse y-axis so that (0, 0) is bottom-left instead of top-left.
    return [(x, y)  # y axis going 
            for y in range(len(lines))
            for x in range(len(lines[y]))
            if lines[y][x] != ' ']


SHAPE_VARIANTS = {
    "L": [
        parse_shape("L ",
                    "L ",
                    "LL"),

        parse_shape("LLL",
                    "L "),

        parse_shape("LL",
                    " L",
                    " L"),

        parse_shape("  L",
                    "LLL"),
    ],
    "J": [
        parse_shape(" J",
                    " J",
                    "JJ"),

        parse_shape("J  ",
                    "JJJ"),

        parse_shape("JJ",
                    "J ",
                    "J "),

        parse_shape("JJJ",
                    "  J"),
    ],
    "I": [
        parse_shape("IIII"),

        parse_shape("I",
                    "I",
                    "I",
                    "I"),
    ],
    "T": [
        parse_shape(" T ",
                    "TTT"),

        parse_shape("T ",
                    "TT",
                    "T "),

        parse_shape("TTT",
                    " T "),

        parse_shape(" T",
                    "TT",
                    " T"),
    ],
    "S": [
        parse_shape("S",
                    "SS",
                    " S"),

        parse_shape(" SS",
                    "SS "),
    ],
    "Z": [
        parse_shape("ZZ ",
                    " ZZ",),

        parse_shape(" Z",
                    "ZZ",
                    "Z "),
    ],
    "O": [
        parse_shape("OO",
                    "OO"),
    ],
}


def score(totems: List[TotemAnswer]) -> float:
    if not totems: return 0
    # Note: no need to check mins, should have (0, 0) in there.
    max_x, max_y = 0, 0
    for totem in totems:
        for x, y in totem.coordinates:
            max_x = max(x, max_x)
            max_y = max(y, max_y)
    side1 = max_x + 1
    side2 = max_y + 1
    return (10 * len(totems) - side1 * side2) * min(side1, side2) / max(side1, side2)


def visualize(totems: List[TotemAnswer]):
    max_x = max(x for totem in totems for x, _ in totem.coordinates)
    max_y = max(y for totem in totems for _, y in totem.coordinates)
    lines = [['.' for _ in range(max_x+1)] for _ in range(max_y+1)]
    for totem in totems:
        for x, y in totem.coordinates:
            lines[y][x] = totem.shape
    lines = lines[::-1]  # y positive is up
    for line in lines:
        print(''.join(line))


def place_above(board, totem: List[CoordinatePair]) -> List[CoordinatePair]:
    totem = list(totem)
    while set(totem) & board:
        totem = [(x, y+1) for x, y in totem]
    return totem


def place_right(board, totem: List[CoordinatePair]) -> List[CoordinatePair]:
    totem = list(totem)
    while set(totem) & board:
        totem = [(x+1, y) for x, y in totem]
    return totem


def greedy_solve(shapes: List[Totem]) -> List[TotemAnswer]:
    assert shapes
    # The first shape _must_ have (0, 0). For now just decide to use the first variant of shapes[0] that has that.
    totems = []
    origin_variant = next(variant for variant in SHAPE_VARIANTS[shapes[0]] if (0, 0) in variant)
    totems.append(TotemAnswer(shape=shapes[0], coordinates=origin_variant))
    shapes = shapes[1:]
    max_x, max_y = max(x for x, _ in origin_variant), max(y for _, y in origin_variant)
    board = set(origin_variant)

    # For each shape variant, try 4 things, pick the best:
    # place it above, place it above pushed right,
    # place it to the right, place it to the right pushed up
    while shapes:
        best_score = 0
        best_shape = None
        best_variant = None
        for shape in shapes:
            for variant in SHAPE_VARIANTS[shape]:
                top_variant = [(x, y+max_y) for x, y in variant]
                right_variant = [(x+max_x, y) for x, y in variant]
                options = [
                    place_above(board, top_variant),
                    place_right(board, top_variant),
                    place_right(board, right_variant),
                    place_above(board, right_variant)
                ]
                for option in options:
                    s = score(totems + [TotemAnswer(shape=shape, coordinates=option)])
                    if s > best_score:
                        best_score = s
                        best_shape = shape
                        best_variant = option
        assert not bool(board & set(best_variant))
        shapes.remove(best_shape)
        board.update(set(best_variant))
        max_x, max_y = max(x for x, _ in best_variant), max(y for _, y in best_variant)
        totems.append(TotemAnswer(shape=best_shape, coordinates=best_variant))
    return totems


class Solver:
    def __init__(self, verbose):
        """
        This method should be use to initialize some variables you will need throughout the challenge.
        """
        self.verbose = verbose

    def get_answer(self, game_message: GameMessage) -> Answer:
        """
        Here is where the magic happens, for now the answer is a single 'I'. I bet you can do better ;)
        """
        start_time = time.time()
        question = game_message.payload
        if self.verbose:
            print("Received Question:", question)
        else:
            print(f"Received question with {len(question.totems)} totems.")
        shapes = [totem.shape for totem in question.totems]

        totems = greedy_solve(shapes)

        print(f"Score: {score(totems)}")
        if self.verbose:
            print("Visually:")
            visualize(totems)
        answer = Answer(totems)
        if self.verbose:
            print("Sending Answer:", answer)
        total_time = time.time() - start_time
        print(f"Took {total_time * 1000:.2f} ms.")
        return answer
