use crate::client::{LocalGameClient, WebSocketGameClient};
use std::env;

mod client;
mod game_interface;
mod greedy_solver;
mod scoring;
mod shape_info;
mod solver;

type SelectedSolver = greedy_solver::GreedySolver;

#[tokio::main]
async fn main() {
    println!(
        "[BUILD ENV]\nPROFILE={}\nOPT_LEVEL={}\nTARGET={}\nTARGET_FEATURE={}\n",
        env!("PROFILE"),
        env!("OPT_LEVEL"),
        env!("TARGET"),
        env!("CARGO_CFG_TARGET_FEATURE"),
    );

    if let Ok(token) = env::var("TOKEN") {
        WebSocketGameClient::<SelectedSolver>::new(token)
            .run()
            .await;
    } else {
        LocalGameClient::<SelectedSolver>::new().run().await;
    }
}
