import os
import time
from typing import List

from game_interface import Answer, GameMessage, Totem, TotemAnswer
import game_logic

import astar_strategy
import greedy_strategy


class Solver:
    def __init__(self):
        verbose = "TOKEN" not in os.environ  # Ideally should pass this down, but I don't get logs then for some reason.
        print(f"Running with verbosity: {verbose}")
        self.verbose = verbose

    def get_answer(self, game_message: GameMessage) -> Answer:
        start_time = time.time()
        question = game_message.payload
        if self.verbose:
            print("Received Question:", question)
        else:
            print(f"Received question with {len(question.totems)} totems.")
        shapes = [totem.shape for totem in question.totems]

        totems = greedy_strategy.solve(shapes)
        #totems = astar_strategy.solve(shapes)
        if self.verbose:
            print("Visually:")
            game_logic.visualize(totems)

        print(f"Score: {game_logic.score(totems)}")
        #if self.verbose:
        #    print(f"Greedy would have given: {game_logic.score(greedy_strategy.solve(shapes))} points.")
        answer = Answer(totems)
        if self.verbose:
            print("Sending Answer:", answer)
        total_time = time.time() - start_time
        print(f"Took {total_time * 1000:.2f} ms.")
        return answer
