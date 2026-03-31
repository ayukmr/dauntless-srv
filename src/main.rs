#[macro_use] extern crate rocket;

mod config;
mod data;
mod frame;
mod meta;
mod nt;
mod web;

use crate::config::Config;
use crate::data::St;
use crate::meta::Meta;

use std::thread;
use std::sync::Arc;

use colored::Colorize;

#[launch]
fn rocket() -> _ {
    let n_cams =
        std::env::var("N_CAMS")
            .ok()
            .and_then(|n| n.parse().ok())
            .unwrap_or(1);

    println!("main: {} [{} camera{}]", "running".green(), n_cams, if n_cams != 1 { "s" } else { "" });

    let mut next_idx = 0;

    let configs =
        Config::load_all()
            .unwrap_or_else(|_| (0..n_cams).map(|_| {
                let cfg = Config::default(next_idx);
                next_idx = cfg.server.camera + 1;
                cfg
            }).collect());

    let meta = Meta::new(n_cams, &configs);

    let states: Vec<_> =
        (0..n_cams)
            .map(|idx| {
                let state = Arc::new(St::new(idx, configs[idx as usize]));

                let st = Arc::clone(&state);
                thread::spawn(move || data::update(&st));

                state
            })
            .collect();

    let sts = states.iter().map(Arc::clone).collect();
    thread::spawn(move || nt::run(sts));

    web::build(meta, states)
}
