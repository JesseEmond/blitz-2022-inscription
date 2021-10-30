import math
from typing import Iterator, List, Optional

from game_interface import CoordinatePair, Totem, TotemAnswer
import game_logic
import shape_info


def tiles_around(pos: CoordinatePair) -> Iterator[CoordinatePair]:
    x, y = pos
    return ((x+dx, y+dy) for dx, dy in [(-1, 0), (1, 0), (0, -1), (0, 1)]
            if x+dx >= 0 and y+dy >= 0)


class Board:
    def __init__(self, size):
        self.size = size
        self.grid = [[False for _ in range(size)] for _ in range(size)]
        self.n_touchpoints = [[0 for _ in range(size)] for _ in range(size)]
        for x in range(size):
            self.n_touchpoints[0][x] += 1
            self.n_touchpoints[-1][x] += 1
        for y in range(size):
            self.n_touchpoints[y][0] += 1
            self.n_touchpoints[y][-1] += 1
        self.n_touchpoints[0][0] += 1  # Give (0,0) a boost to ensure we set it.
        self.totems = []
        self.smallest_unset_at_x = [0 for _ in range(size)]

    def mark(self, totem: List[CoordinatePair]) -> None:
        for x, y in totem:
            self.grid[y][x] = True
            if y == self.smallest_unset_at_x[x]:
                new_unset_y = y+1
                while new_unset_y < self.size and self.grid[new_unset_y][x]:
                    new_unset_y += 1
                self.smallest_unset_at_x[x] = new_unset_y
            for nx, ny in tiles_around((x, y)):
                if nx < self.size and ny < self.size:
                    self.n_touchpoints[ny][nx] += 1

    def make_choice(self, shape: Totem, totem: List[CoordinatePair]) -> None:
        self.totems.append(TotemAnswer(shape=shape, coordinates=totem))
        self.mark(totem)

    def fits(self, totem: List[CoordinatePair]):
        """Would 'totem' fit in the grid?"""
        return all(self.is_empty_tile(pos) for pos in totem)

    def is_empty_tile(self, pos: CoordinatePair):
        x, y = pos
        return 0 <= x < self.size and 0 <= y < self.size and not self.grid[y][x]

    def touchpoints(self, totem: List[CoordinatePair]) -> int:
        """Number of placed pieces (or bounds) that this totem would touch."""
        return sum(self.n_touchpoints[y][x] for x, y in totem)



def place_w_delta(
    board: Board, totem: List[CoordinatePair], delta: CoordinatePair
) -> Optional[List[CoordinatePair]]:
    """Move 'totem' by 'delta' until it fits on the board."""
    dx, dy = delta
    totem = [[x, y] for x, y in totem]  # list for in-place edits.
    while not board.fits(totem):
        for pos in totem:
            pos[0] += dx
            pos[1] += dy
            if pos[0] >= board.size or pos[1] >= board.size:
                return None
    return [tuple(pos) for pos in totem]


def place_above(board: Board, totem: List[CoordinatePair]) -> Optional[List[CoordinatePair]]:
    """Move 'totem' up until it fits on the board."""
    return place_w_delta(board, totem, (0, 1))


def place_right(board: Board, totem: List[CoordinatePair]) -> Optional[List[CoordinatePair]]:
    """Move 'totem' right until it fits on the board."""
    return place_w_delta(board, totem, (1, 0))


def try_fit(board: Board, shapes: List[Totem]) -> Optional[TotemAnswer]:
    """Try to pack 'shapes' in a 'side'x'side' grid."""
    shapes = list(shapes)
    totems = []
    while shapes:
        best_shape = None
        best_touchpoints = 0
        best_variant = None
        for shape in set(shapes):
            for variant, (w, h) in shape_info.variants_with_dims(shape):
                for dx in range(board.size - w + 1):
                    # Simulate a piece falling down
                    min_height = max(board.smallest_unset_at_x[x+dx]-y for x, y in variant)
                    placed = place_above(board, [(x+dx, y+min_height) for x, y in variant])
                    if placed:
                        touchpoints = board.touchpoints(placed)
                        if touchpoints > best_touchpoints:
                            best_shape = shape
                            best_variant = placed
                            best_touchpoints = touchpoints
        if best_shape is None:
            return None
        shapes.remove(best_shape)
        board.make_choice(best_shape, best_variant)
    return board.totems



def solve(shapes: List[Totem]) -> List[TotemAnswer]:
    n_squares = len(shapes) * 4
    side = math.ceil(math.sqrt(n_squares))
    while True:
        fit = try_fit(Board(size=side), shapes)
        print(f"Fit on a {side}x{side} board?  {bool(fit)}")
        if fit:
            return fit
        side += 1