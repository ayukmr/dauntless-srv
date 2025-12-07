#[macro_use] extern crate rocket;

use dauntless::{Config, Tag};

use std::thread;
use std::sync::{Arc, RwLock};

use std::io;
use std::io::Write;
use std::time::Instant;

use rocket::State;
use rocket::fs::FileServer;

use rocket::serde::json::Json;
use serde::Serialize;

use ndarray::Array2;
use opencv::{core, videoio, imgproc};
use opencv::prelude::*;

#[derive(Serialize)]
pub struct Data {
    pub tags: Vec<Tag>,
    pub frame: Vec<Vec<f32>>,
    pub mask: Vec<Vec<f32>>,
}

#[get("/tags")]
fn tags(state: &State<Arc<RwLock<Data>>>) -> Json<Vec<Tag>> {
    Json(state.read().unwrap().tags.clone())
}

#[get("/frame")]
fn frame(state: &State<Arc<RwLock<Data>>>) -> Json<Vec<Vec<f32>>> {
    Json(state.read().unwrap().frame.clone())
}

#[get("/mask")]
fn mask(state: &State<Arc<RwLock<Data>>>) -> Json<Vec<Vec<f32>>> {
    Json(state.read().unwrap().mask.clone())
}

#[get("/config")]
fn get_config() -> Json<Config> {
    Json(dauntless::get_config().clone())
}

#[post("/config", data = "<config>")]
fn set_config(config: Json<Config>) {
    dauntless::set_config(*config);
}

#[post("/config/reset")]
fn reset_config() -> Json<Config> {
    let config = Config::default();

    dauntless::set_config(config);

    Json(config)
}

#[launch]
fn rocket() -> _ {
    let state = Arc::new(
        RwLock::new(
            Data {
                tags: Vec::new(),
                frame: Vec::new(),
                mask: Vec::new(),
            },
        ),
    );

    let state_cln = Arc::clone(&state);
    thread::spawn(move || update(&state_cln));

    dauntless::set_config(Config::default());

    rocket::build()
        .manage(state)
        .mount("/", FileServer::from("./www"))
        .mount("/api", routes![tags, frame, mask, get_config, set_config, reset_config])
}

fn update(state: &Arc<RwLock<Data>>) {
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap();

    let mut last = Instant::now();
    let mut fps = 0.0;
    let mut tick = 0;

    loop {
        let mut frame = Mat::default();
        cam.read(&mut frame).unwrap();

        if frame.empty() {
            continue;
        }

        let now = Instant::now();
        let dt = now.duration_since(last).as_secs_f64();

        last = now;

        if dt > 0.0 {
            fps = 0.9 * fps + 0.1 * (1.0 / dt);
        }

        if tick % 10 == 0 {
            print!("\r{} fps", fps as i32);
            io::stdout().flush().unwrap();
        }
        tick += 1;

        let mut light = Mat::default();

        imgproc::cvt_color(
            &frame,
            &mut light,
            imgproc::COLOR_BGR2GRAY,
            0,
            core::AlgorithmHint::ALGO_HINT_DEFAULT,
        ).unwrap();

        let h = light.rows();
        let w = light.cols();

        let scale = 400.0 / i32::max(w, h) as f32;

        let sw = (w as f32 * scale) as i32;
        let sh = (h as f32 * scale) as i32;

        let mut resized = Mat::default();

        imgproc::resize(
            &light,
            &mut resized,
            core::Size::new(sw, sh),
            0.0,
            0.0,
            imgproc::INTER_LINEAR,
        ).unwrap();

        let data = Array2::from_shape_vec(
            (sh as usize, sw as usize),
            resized.data_bytes().unwrap().to_vec(),
        ).unwrap().mapv(|l| l as f32) / 255.0;

        let mut s = state.write().unwrap();

        s.frame = to_data(&resized);

        let (mask, tags) = dauntless::tags2(data);

        let h = mask.dim().0;

        let a = mask.mapv(|b| if b { 255u8 } else { 0u8 });
        let x = a.as_standard_layout();
        let slice = x.as_slice().unwrap();

        let mat = Mat::from_slice(&slice).unwrap().reshape(1, h as i32).unwrap().clone_pointee();

        s.mask = to_data(&mat);

        s.tags = tags;
    }
}

fn to_data(img: &Mat) -> Vec<Vec<f32>> {
    let h = img.rows();
    let w = img.cols();

    let mut resized = Mat::default();

    let scale = 200.0 / i32::max(w, h) as f32;
    let sw = (w as f32 * scale) as i32;
    let sh = (h as f32 * scale) as i32;

    imgproc::resize(
        img,
        &mut resized,
        core::Size::new(sw, sh),
        0.0,
        0.0,
        imgproc::INTER_LINEAR,
    ).unwrap();

    let data = Array2::from_shape_vec(
        (sh as usize, sw as usize),
        resized.data_bytes().unwrap().to_vec(),
    ).unwrap().mapv(|l| l as f32) / 255.0;

    data
        .rows()
        .into_iter()
        .map(|row| row.to_vec())
        .collect()
}
