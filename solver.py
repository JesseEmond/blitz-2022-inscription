from game_interface import Answer, GameMessage, TotemAnswer


class Solver:
    def __init__(self):
        """
        This method should be use to initialize some variables you will need throughout the challenge.
        """
        pass

    def get_answer(self, game_message: GameMessage) -> Answer:
        """
        Here is where the magic happens, for now the answer is a single 'I'. I bet you can do better ;)
        """
        question = game_message.payload
        print("Received Question:", question)

        totems = [
            TotemAnswer(shape="I", coordinates=[(0, 0), (1, 0), (2, 0), (3, 0)])
        ]

        answer = Answer(totems)
        print("Sending Answer:", answer)
        return answer
