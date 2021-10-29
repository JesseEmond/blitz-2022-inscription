from typing import Iterator, List, Set

import astar

import game_logic
import shape_info
from game_interface import CoordinatePair, Totem, TotemAnswer


def move_totem_to(
    totem: List[CoordinatePair], pos: CoordinatePair
) -> List[CoordinatePair]:
    """Move the totem such that the first coord is at 'pos'."""
    px, py = pos
    tx, ty = totem[0]
    dx, dy = px - tx, py - ty
    return [(x + dx, y + dy) for x, y in totem]


class SearchNode:
    def __init__(
        self,
        shapes_left: List[Totem],
        max_x: int,
        max_y: int,
        used: Set[CoordinatePair],
        holes: Set[CoordinatePair],
        totems: List[TotemAnswer],
    ):
        self.shapes_left = shapes_left
        self.max_x = max_x
        self.max_y = max_y
        self.used = used
        self.holes = holes
        self.totems = totems

    def with_totem(self, totem: TotemAnswer) -> "SearchNode":
        new_shapes_left = list(self.shapes_left)
        new_shapes_left.remove(totem.shape)
        holes = set(self.holes)
        max_x = max(x for x, _ in totem.coordinates)
        if max_x > self.max_x:
            holes |= {
                (x, y) for x in range(self.max_x, max_x + 1) for y in range(self.max_y)
            }
        max_x = max(self.max_x, max_x)
        max_y = max(y for _, y in totem.coordinates)
        if max_y > self.max_y:
            holes |= {
                (x, y) for x in range(self.max_x) for y in range(self.max_y, max_y + 1)
            }
        max_y = max(self.max_y, max_y)
        return SearchNode(
            shapes_left=new_shapes_left,
            max_x=max_x,
            max_y=max_y,
            used=self.used & set(totem.coordinates),
            holes=holes - set(totem.coordinates),
            totems=self.totems + [totem],
        )

    def shape_fits(self, shape: List[CoordinatePair]) -> bool:
        intersects = bool(set(shape) & self.used)
        negatives = any(x < 0 or y < 0 for x, y in shape)
        return not intersects and not negatives

    def is_goal(self):
        return not self.shapes_left

    def score(self):
        # Negate to maximize score.
        return -game_logic.score(self.totems, maxima=(self.max_x, self.max_y))

    def neighbors(self) -> Iterator["SearchNode"]:
        for shape in set(self.shapes_left):
            for variant, (w, h) in shape_info.variants_with_dims(shape):
                # First piece _must_ have (0, 0).
                if not self.totems:
                    if (0, 0) not in variant:
                        continue
                    yield self.with_totem(TotemAnswer(shape=shape, coordinates=variant))
                else:
                    # Check if the piece can fit in any of the unused spots.
                    for hole in self.holes:
                        new_variant = move_totem_to(variant, hole)
                        if self.shape_fits(new_variant):
                            yield self.with_totem(
                                TotemAnswer(shape=shape, coordinates=new_variant)
                            )

                    # Check borders of our used area
                    for x in range(self.max_x):  # along the top
                        for y in range(self.max_y + 1, self.max_y + h + 1):
                            new_variant = move_totem_to(variant, (x, y))
                            if self.shape_fits(new_variant):
                                yield self.with_totem(
                                    TotemAnswer(shape=shape, coordinates=new_variant)
                                )
                    for y in range(self.max_y):  # along the right
                        for x in range(self.max_x + 1, self.max_x + w + 1):
                            new_variant = move_totem_to(variant, (x, y))
                            if self.shape_fits(new_variant):
                                yield self.with_totem(
                                    TotemAnswer(shape=shape, coordinates=new_variant)
                                )


def is_goal_reached(current: SearchNode, _) -> bool:
    return current.is_goal()


def distance(from_: SearchNode, to: SearchNode) -> float:
    print(f"from: {from_.score()}  to: {to.score()}")
    return to.score() - from_.score()


def neighbors(current: SearchNode) -> Iterator[SearchNode]:
    return current.neighbors()


def solve(shapes: List[Totem]) -> List[TotemAnswer]:
    start = SearchNode(
        shapes_left=shapes, max_x=0, max_y=0, used=set(), holes=set(), totems=[]
    )
    path = astar.find_path(
        start=start,
        goal=None,
        neighbors_fnct=neighbors,
        reversePath=True,
        # heuristic_cost_estimate_fnct=heuristic,
        distance_between_fnct=distance,
        is_goal_reached_fnct=is_goal_reached,
    )
    goal = next(path)
    return goal.totems
