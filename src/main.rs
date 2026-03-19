#[macro_use] extern crate rocket;

use crate::data::St;

use dauntless::{Config, Detector};

use std::thread;
use std::sync::Arc;

const FOV: f32 = 56_f32;

mod consts;
mod data;
mod frame;
mod nt;
mod web;

#[launch]
fn rocket() -> _ {
    let fov = 2.0 * (FOV.to_radians() * 0.5).tan().atan();

    let detector = Detector::new(
        Config {
            fov_rad: fov,
            hyst_low: 0.025,
            hyst_high: 0.05,
            filter_ratios: true,
            filter_angles: true,
            filter_enclosed: false
        },
    );

    let state = Arc::new(St::new(detector));

    let st = Arc::clone(&state);
    thread::spawn(move || data::update(&st));
    let st = Arc::clone(&state);
    thread::spawn(move || nt::run(&st));

    web::build(state)
}
