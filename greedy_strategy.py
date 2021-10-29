from typing import List

from game_interface import CoordinatePair, Totem, TotemAnswer
import game_logic
import shape_info


class Board:
    def __init__(self, size=256):
        self.size = size
        self.grid = [[False for _ in range(size)] for _ in range(size)]
        self.max_x = 0
        self.max_y = 0
        # Smallest unused coordinate at a given x (or y) column (or row).
        self.unused_x = [0 for _ in range(size)]
        self.unused_y = [0 for _ in range(size)]
        self.n_totems = 0

    def mark(self, totem: List[CoordinatePair]):
        for x, y in totem:
            self.grid[y][x] = True
            if y == self.unused_x[x]:
                self.unused_x[x] = next(new_y for new_y in range(y + 1, self.size) if not self.grid[new_y][x])
            if x == self.unused_y[y]:
                self.unused_y[y] = next(new_x for new_x in range(x + 1, self.size) if not self.grid[y][new_x])
            self.max_x = max(self.max_x, x)
            self.max_y = max(self.max_y, y)
        self.n_totems += 1

    def fits(self, totem: List[CoordinatePair]):
        return not any(self.grid[y][x] for x, y in totem)

    def earliest_unused_right(self, coord: CoordinatePair) -> CoordinatePair:
        x, y = coord
        return (self.unused_y[y], y)

    def earliest_unused_above(self, coord: CoordinatePair) -> CoordinatePair:
        x, y = coord
        return (x, self.unused_x[x])

    def tentative_score(self, totem: List[CoordinatePair]) -> float:
        max_x = max(self.max_x, max(x for x, _ in totem))
        max_y = max(self.max_y, max(y for _, y in totem))
        return game_logic.fast_score(self.n_totems + 1, (max_x, max_y))


def place_w_delta(board: Board, totem: List[CoordinatePair], delta: CoordinatePair) -> List[CoordinatePair]:
    """Move 'totem' by 'delta' until it fits on the board."""
    dx, dy = delta
    totem = [[x, y] for x, y in totem]  # list for in-place edits.
    while not board.fits(totem):
        for pos in totem:
            pos[0] += dx
            pos[1] += dy
    return [tuple(pos) for pos in totem]


def place_above(board: Board, totem: List[CoordinatePair]) -> List[CoordinatePair]:
    """Move 'totem' up until it fits on the board."""
    delta_y = board.earliest_unused_above(totem[0])[1] - totem[0][1]
    #totem = [(x, y+delta_y) for x, y in totem]
    return place_w_delta(board, totem, (0, 1))


def place_right(board: Board, totem: List[CoordinatePair]) -> List[CoordinatePair]:
    """Move 'totem' right until it fits on the board."""
    delta_x = board.earliest_unused_right(totem[0])[0] - totem[0][0]
    #totem = [(x+delta_x, y) for x, y in totem]
    return place_w_delta(board, totem, (1, 0))


def solve(shapes: List[Totem]) -> List[TotemAnswer]:
    assert shapes
    # The first shape _must_ have (0, 0). For now just decide to use the first variant of shapes[0] that has that.
    totems = []
    origin_variant = next(variant for variant in shape_info.variants(shapes[0]) if (0, 0) in variant)
    totems.append(TotemAnswer(shape=shapes[0], coordinates=origin_variant))
    shapes = shapes[1:]
    board = Board()
    board.mark(origin_variant)

    # For each shape variant, try 4 things, pick the best:
    # place it above, place it above pushed right,
    # place it to the right, place it to the right pushed up
    while shapes:
        best_score = None
        best_shape = None
        best_variant = None
        for shape in set(shapes):
            for variant, (w, h) in shape_info.variants_with_dims(shape):
                top_variant = [(x, y+board.max_y+1-h) for x, y in variant]
                right_variant = [(x+board.max_x+1-w, y) for x, y in variant]
                options = [
                    place_above(board, top_variant),
                    place_right(board, top_variant),
                    # place_right with one above top_variant?
                    place_right(board, right_variant),
                    place_above(board, right_variant)
                    # place_above with one right of right_variant?
                ]
                for option in options:
                    if any(x < 0 or y < 0 for x, y in option):
                        continue
                    score = board.tentative_score(option)
                    if best_score is None or score > best_score:
                        best_score = score
                        best_shape = shape
                        best_variant = option
        assert best_variant is not None
        shapes.remove(best_shape)
        board.mark(best_variant)
        totems.append(TotemAnswer(shape=best_shape, coordinates=best_variant))
    return totems