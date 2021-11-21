use crate::client::{LocalGameClient, WebSocketGameClient};
use crate::solver::Solver;
use std::env;
extern crate dlx;

mod client;
mod exact_solver;
mod game_interface;
mod scoring;
mod shape_info;
mod solver;

mod dlx_solver;
mod max_rects;
mod rect_inventory;
mod rect_packing;
mod subset_sum;

#[tokio::main]
async fn main() {
    println!(
        "[BUILD ENV]\nPROFILE={}\nOPT_LEVEL={}\nTARGET={}\nTARGET_FEATURE={}\n",
        env!("PROFILE"),
        env!("OPT_LEVEL"),
        env!("TARGET"),
        env!("CARGO_CFG_TARGET_FEATURE"),
    );

    let solver = Solver::new();
    if let Ok(token) = env::var("TOKEN") {
        WebSocketGameClient::new(solver, token).run().await;
    } else {
        LocalGameClient::new(solver).run().await;
    }
}
