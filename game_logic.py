from typing import List, Optional

from game_interface import CoordinatePair, TotemAnswer


def score(totems: List[TotemAnswer], maxima: Optional[CoordinatePair] = None) -> float:
    if not totems: return 0
    # Note: no need to check mins, should have (0, 0) in there.
    if maxima:
        max_x, max_y = maxima
    else:
        max_x, max_y = 0, 0
        for totem in totems:
            for x, y in totem.coordinates:
                max_x = max(x, max_x)
                max_y = max(y, max_y)
    side1 = max_x + 1
    side2 = max_y + 1
    return (10 * len(totems) - side1 * side2) * min(side1, side2) / max(side1, side2)


def visualize(totems: List[TotemAnswer]):
    assert totems
    max_x = max(x for totem in totems for x, _ in totem.coordinates)
    max_y = max(y for totem in totems for _, y in totem.coordinates)
    lines = [['.' for _ in range(max_x+1)] for _ in range(max_y+1)]
    for totem in totems:
        for x, y in totem.coordinates:
            lines[y][x] = totem.shape
    lines = lines[::-1]  # y positive is up
    for line in lines:
        print(''.join(line))