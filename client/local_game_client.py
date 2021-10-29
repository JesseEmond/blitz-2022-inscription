import os
import random

from game_interface import GameMessage, Question, TotemQuestion
from solver import Solver


class LocalGameClient:
    def __init__(self, solver: Solver):
        self.solver = solver

    async def run(self):
        print("[Running in local mode]")
        if "SEED" in os.environ:
            random.seed(int(os.environ["SEED"]))
        num_shapes = int(os.environ.get("NUM_SHAPES", "8"))
        SHAPES = ["I", "O", "J", "L", "S", "Z", "T"]
        questions = [TotemQuestion(shape=random.choice(SHAPES)) for _ in range(num_shapes)]
        game_message: GameMessage = GameMessage(
            tick=1, payload=Question(totems=questions)
        )
        self.solver.get_answer(game_message)
