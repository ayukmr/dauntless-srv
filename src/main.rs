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

#[launch]
fn rocket() -> _ {
    let n = 2;

    let configs =
        Config::load_all()
            .unwrap_or_else(|_| (0..n).map(Config::default).collect());

    let meta = Meta::new(&configs);

    let states: Vec<_> =
        (0..n)
            .map(|idx| {
                let state = Arc::new(St::new(idx, configs[idx as usize]));

                let st = Arc::clone(&state);
                thread::spawn(move || data::update(&st));

                state
            })
            .collect();

    let sts = states.iter().map(|st| Arc::clone(&st)).collect();
    thread::spawn(move || nt::run(sts));

    web::build(meta, states)
}
