from typing import List

from game_interface import CoordinatePair, Totem, TotemAnswer
import game_logic
import shape_info


def place_above(board, totem: List[CoordinatePair]) -> List[CoordinatePair]:
    """Move 'totem' up until it fits on the board."""
    totem = list(totem)
    while set(totem) & board:
        totem = [(x, y+1) for x, y in totem]
    return totem


def place_right(board, totem: List[CoordinatePair]) -> List[CoordinatePair]:
    """Move 'totem' right until it fits on the board."""
    totem = list(totem)
    while set(totem) & board:
        totem = [(x+1, y) for x, y in totem]
    return totem


def solve(shapes: List[Totem]) -> List[TotemAnswer]:
    assert shapes
    # The first shape _must_ have (0, 0). For now just decide to use the first variant of shapes[0] that has that.
    totems = []
    origin_variant = next(variant for variant in shape_info.variants(shapes[0]) if (0, 0) in variant)
    totems.append(TotemAnswer(shape=shapes[0], coordinates=origin_variant))
    shapes = shapes[1:]
    max_x, max_y = max(x for x, _ in origin_variant), max(y for _, y in origin_variant)
    board = set(origin_variant)

    # For each shape variant, try 4 things, pick the best:
    # place it above, place it above pushed right,
    # place it to the right, place it to the right pushed up
    while shapes:
        best_score = None
        best_shape = None
        best_variant = None
        for shape in shapes:
            for variant, (w, h) in shape_info.variants_with_dims(shape):
                top_variant = [(x, y+max_y+1-h) for x, y in variant]
                right_variant = [(x+max_x+1-w, y) for x, y in variant]
                options = [
                    place_above(board, top_variant),
                    place_right(board, top_variant),
                    # place_right with one above top_variant?
                    place_right(board, right_variant),
                    place_above(board, right_variant)
                    # place_above with one right of right_variant?
                ]
                for option in options:
                    new_max_x, new_max_y = max(max_x, max(x for x, _ in option)), max(max(y for _, y in option), max_y)
                    score = game_logic.score(totems + [TotemAnswer(shape=shape, coordinates=option)], maxima=(new_max_x, new_max_y))
                    if best_score is None or score > best_score:
                        best_score = score
                        best_shape = shape
                        best_variant = option
        assert best_variant is not None
        assert not bool(board & set(best_variant))
        shapes.remove(best_shape)
        board.update(set(best_variant))
        max_x, max_y = max(max_x, max(x for x, _ in best_variant)), max(max(y for _, y in best_variant), max_y)
        totems.append(TotemAnswer(shape=best_shape, coordinates=best_variant))
    return totems