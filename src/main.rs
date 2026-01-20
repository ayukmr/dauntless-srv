#[macro_use] extern crate rocket;

use crate::data::Data;

use std::thread;

use arc_swap::ArcSwap;
use std::sync::Arc;

mod consts;
mod frame;
mod data;
mod web;
mod nt;

#[launch]
fn rocket() -> _ {
    let state = Arc::new(
        ArcSwap::from_pointee(
            Data {
                fps: None,
                tags: Vec::new(),
                frame: None,
                mask: None,
            },
        ),
    );

    let st = Arc::clone(&state);
    thread::spawn(move || data::update(&st));
    let st = Arc::clone(&state);
    thread::spawn(move || nt::run(&st));

    web::build(state)
}
