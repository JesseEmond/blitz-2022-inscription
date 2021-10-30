import os
import time
from typing import List

import astar_strategy
import game_logic
import greedy_strategy
import js_strategy
from game_interface import Answer, GameMessage, Totem, TotemAnswer


def validate(totems: List[TotemAnswer]):
    print("Verifying answer validity...  ", end="")
    coords = set()
    ok = True
    for totem in totems:
        for x, y in totem.coordinates:
            if (x, y) in coords:
                print("INVALID")
                print(f"Duplicate coords: {(x, y)}")
                ok = False
            if x < 0 or y < 0:
                print("INVALID")
                print(f"Negative coords: {(x, y)}")
                ok = False
            coords.add((x, y))
    if ok:
        print("OK")


class Solver:
    def __init__(self):
        verbose = (
            "TOKEN" not in os.environ
        )  # Ideally should pass this down, but I don't get logs then for some reason.
        print(f"Running with verbosity: {verbose}")
        self.verbose = verbose

    def get_answer(self, game_message: GameMessage) -> Answer:
        start_time = time.time()
        question = game_message.payload
        if self.verbose and len(question.totems) <= 25:
            print("Received Question:", question)
        else:
            print(f"Received question with {len(question.totems)} totems.")
        shapes = [totem.shape for totem in question.totems]

        totems = js_strategy.solve(shapes)

        if self.verbose:
            print("Visually:")
            game_logic.visualize(totems)

        print(f"Score: {game_logic.score(totems)}")
        answer = Answer(totems)
        if self.verbose:
            validate(totems)
        if self.verbose and len(totems) <= 25:
            print("Sending Answer:", answer)
        total_time = time.time() - start_time
        print(f"Took {total_time * 1000:.2f} ms.")
        if self.verbose:
            solvers = {
                "Greedy": greedy_strategy,
                "JS": js_strategy,
            }
            for key, value in solvers.items():
                print(
                    f"{key} would have given: {game_logic.score(value.solve(shapes))} points."
                )
        return answer
