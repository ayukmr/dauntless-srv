#[macro_use] extern crate rocket;

mod config;
mod data;
mod meta;
mod nt;
mod state;
mod web;

use crate::state::States;

use colored::Colorize;

#[launch]
async fn rocket() -> _ {
    let n_cams =
        std::env::var("N_CAMS")
            .ok()
            .and_then(|n| n.parse().ok())
            .unwrap_or(1);

    println!("main: {} [{} camera{}]", "running".green(), n_cams, if n_cams != 1 { "s" } else { "" });

    let states = States::new(n_cams);

    let sts = states.states.iter().map(|s| s.clone()).collect();
    let ntfy = states.notify.clone();

    tokio::spawn(nt::run(sts, ntfy));

    web::build(states)
}
