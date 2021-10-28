#!/usr/bin/env python

import asyncio
import os

from client.local_game_client import LocalGameClient
from client.websocket_game_client import WebSocketGameClient
from solver import Solver


async def run():
    is_local = "TOKEN" not in os.environ
    solver = Solver(verbose=not is_local)
    if is_local:
        await LocalGameClient(solver).run()
    else:
        await WebSocketGameClient(solver).run()


if __name__ == "__main__":
    asyncio.get_event_loop().run_until_complete(run())
