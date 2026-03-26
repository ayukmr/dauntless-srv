#[macro_use] extern crate rocket;

mod config;
mod data;
mod frame;
mod meta;
mod nt;
mod web;

use crate::data::St;

use std::thread;
use std::sync::Arc;

#[launch]
fn rocket() -> _ {
    let state = Arc::new(St::new());

    let st = Arc::clone(&state);
    thread::spawn(move || data::update(&st));
    let st = Arc::clone(&state);
    thread::spawn(move || nt::run(&st));

    web::build(state)
}
