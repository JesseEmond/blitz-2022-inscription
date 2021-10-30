use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use crate::game_interface::{GameMessage};
use crate::solver::Solver;

pub struct WebSocketGameClient {
    solver: Solver,
    uri: String,
    token: String,
}

impl WebSocketGameClient {
    pub fn new(solver: Solver, token: String) -> Self {
        WebSocketGameClient { solver, uri: "ws://127.0.0.1:8765".to_string(), token }
    }

    pub async fn run(&self) {
        let (mut stream, _resp) = connect_async(&self.uri).await.expect("Could not connect to the game");

        let registration = json!({"type": "REGISTER", "token": self.token.clone()});
        stream.send(Message::text(registration.to_string())).await.expect("Could not send to the server");

        loop {
            if let Some(raw_message) = stream.next().await {
                let raw_message = raw_message.expect("Could not read the server's message");
                let message_text = raw_message.to_text().expect("The server sent a message that was not valid UTF-8");

                if message_text.is_empty() {
                    eprintln!("The server did not respond to our registration request");
                    break;
                }

                let parsed: Value = serde_json::from_str(&message_text).expect("The server sent an invalid JSON payload");

                if parsed["type"] == "ERROR" {
                    eprintln!("{}", parsed);
                    break;
                }

                let game_message: GameMessage = serde_json::from_str(&message_text).expect("The server sent a game message that could not be parsed");
                let answer = self.solver.get_answer(&game_message).expect("There was an error in the solver's code!");

                let response = json!({"type": "COMMAND", "tick": game_message.tick, "actions": answer});
                stream.send(Message::Text(response.to_string())).await.expect("Could not send to the server");
            }
        }
    }
}
