use std::env;

mod client;

use application::hybrid_solver;
use client::{LocalGameClient, WebSocketGameClient};

type SelectedSolver = hybrid_solver::HybridSolver;

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
