use std::env;

mod bitgrid;
mod client;
mod dlx;
mod game_interface;
mod greedy_solver;
mod scoring;
mod shape_info;
mod solver;

use client::{LocalGameClient, WebSocketGameClient};

//type SelectedSolver = greedy_solver::GreedySolver;
type SelectedSolver = dlx::DlxSolver;

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
