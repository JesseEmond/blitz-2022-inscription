use std::env;
use crate::client::{LocalGameClient, WebSocketGameClient};
use crate::solver::Solver;

mod game_interface;
mod client;
mod solver;
mod shape_info;

#[tokio::main]
async fn main() {
    let solver = Solver::new();
    if let Ok(token) = env::var("TOKEN") {
        WebSocketGameClient::new(solver, token).run().await;
    } else {
        LocalGameClient::new(solver).run().await;
    }
}
