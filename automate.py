# Script to launch a game every few minutes, due to the luck nature of some of the problems
import requests
import time

def get_last_game_score(access_token):
    cookies = {"access_token": access_token}
    json = {
        "variables": {},
        "query": "{ blitz_tasks(where: {type: {_eq: \"game\"} }, order_by: {id: desc}, limit: 1) {id state errors game {blitz_gamescore_aggregate {aggregate {max {score}}}}}}"
    }
    r = requests.post("https://api.blitz.codes/graphql", json=json, cookies=cookies)
    return r.json()


def launch_game():
    cookies = {"access_token": access_token}
    r = requests.post("https://api.blitz.codes/inscriptionchallenge", cookies=cookies)
    assert r.status_code == 201


access_token = input("access_token=")
while True:
    resp = get_last_game_score(access_token)
    blitz_task = resp['data']['blitz_tasks'][0]
    if blitz_task["game"] is not None:
        score = blitz_task['game']['blitz_gamescore_aggregate']['aggregate']['max']['score']
        print(f"Last game: state={blitz_task['state']} errors={blitz_task['errors'] or 'none'} score={score}")
    else:
        print(f"Game information not available: {blitz_task}")
    launch_game()
    wait_minutes = 3
    print(f"Waiting {wait_minutes} minutes...")
    time.sleep(wait_minutes * 60)
    print()