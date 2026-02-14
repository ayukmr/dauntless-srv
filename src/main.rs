#[macro_use] extern crate rocket;

use crate::data::{Data, St};
use crate::consts::CROP;

use dauntless::{Config, Detector};

use std::thread;
use std::time::SystemTime;

use std::sync::Arc;

const FOV: f32 = 48_f32;

mod consts;
mod data;
mod frame;
mod nt;
mod web;

#[launch]
fn rocket() -> _ {
    let fov = 2.0 * ((FOV.to_radians() * 0.5).tan() / CROP).atan();

    let detector = Detector::new(
        Config {
            fov_rad: fov,
            harris_k: 0.04,
            harris_thresh: 0.001,
            hyst_low: 0.1,
            hyst_high: 0.2,
            ..Config::default()
        },
    );

    let data = Data {
        fps: None,
        tags: Vec::new(),
        frame: None,
        mask: None,
        time: SystemTime::now(),
    };

    let state = Arc::new(St::new(detector, data));

    let st = Arc::clone(&state);
    thread::spawn(move || data::update(&st));
    let st = Arc::clone(&state);
    thread::spawn(move || nt::run(&st));

    web::build(state)
}
