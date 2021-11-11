#![allow(dead_code)]

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::env;

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
        "[BUILD ENV]\nPROFILE={}\nOPT_LEVEL={}\nPGO_USE={}\nTARGET={}\nTARGET_FEATURE={}\n",
        env!("PROFILE"),
        env!("OPT_LEVEL"),
        env!("PGO_USE"),
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
