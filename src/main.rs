#[macro_use] extern crate rocket;

use dauntless::{Config, Tag};

use std::io;
use std::io::Write;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use std::thread;

use rocket::State;
use rocket::fs::FileServer;
use rocket::serde::json::Json;
use rocket::http::ContentType;

use ndarray::Array2;
use opencv::{core, imgcodecs, imgproc, videoio};
use opencv::prelude::*;

const SCALE: i32 = 4;

struct Data {
    pub fps: Option<f32>,
    pub tags: Vec<Tag>,
    pub frame: Option<Arc<[u8]>>,
    pub mask: Option<Arc<[u8]>>,
}

pub fn encode(mat: &Mat) -> Vec<u8> {
    let params = core::Vector::<i32>::from_iter([
        imgcodecs::IMWRITE_JPEG_QUALITY, 75,
    ]);

    let mut buf = core::Vector::<u8>::new();
    imgcodecs::imencode(".jpg", mat, &mut buf, &params).unwrap();

    buf.to_vec().into()
}

#[get("/fps")]
fn fps(state: &State<Arc<RwLock<Data>>>) -> Json<Option<f32>> {
    Json(state.read().unwrap().fps)
}

#[get("/tags")]
fn tags(state: &State<Arc<RwLock<Data>>>) -> Json<Vec<Tag>> {
    Json(state.read().unwrap().tags.clone())
}

#[get("/frame")]
fn frame(state: &State<Arc<RwLock<Data>>>) -> Option<(ContentType, Arc<[u8]>)> {
    let bytes = state.read().unwrap().frame.as_ref()?.clone();
    Some((ContentType::JPEG, bytes))
}

#[get("/mask")]
fn mask(state: &State<Arc<RwLock<Data>>>) -> Option<(ContentType, Arc<[u8]>)> {
    let bytes = state.read().unwrap().mask.as_ref()?.clone();
    Some((ContentType::JPEG, bytes))
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
                fps: None,
                tags: Vec::new(),
                frame: None,
                mask: None,
            },
        ),
    );

    let state_cln = Arc::clone(&state);
    thread::spawn(move || update(&state_cln));

    dauntless::set_config(Config {
        harris_k: 0.001,
        harris_thresh: 0.001,
        hyst_high: 0.2,
        filter_enclosed: false,
        ..Config::default()
    });

    rocket::build()
        .manage(state)
        .mount("/", FileServer::from("./www"))
        .mount("/api", routes![
            fps,
            tags,
            frame,
            mask,
            get_config,
            set_config,
            reset_config,
        ])
}

fn update(state: &Arc<RwLock<Data>>) {
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap();

    let mut last = Instant::now();
    let mut fps = None;
    let mut tick = 0;

    loop {
        let mut frame = Mat::default();
        cam.read(&mut frame).unwrap();

        if frame.empty() {
            continue;
        }

        let now = Instant::now();
        let dt = now.duration_since(last).as_secs_f32();

        last = now;

        if dt > 0.0 {
            if let Some(last) = fps {
                fps = Some(0.9 * last + 0.1 * (1.0 / dt));
            } else {
                fps = Some(1.0 / dt);
            }
        }

        if tick % 10 == 0 {
            print!("\r{} fps", fps.unwrap() as i32);
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

        let w = light.cols();
        let h = light.rows();

        let cw = w / 2;
        let ch = h / 2;
        let x = (w - cw) / 2;
        let y = (h - ch) / 2;

        let roi = core::Rect::new(x, y, cw, ch);
        let cropped = Mat::roi(&light, roi).unwrap();

        let resized = resize(&cropped.clone_pointee(), 400);

        let sw = resized.cols();
        let sh = resized.rows();

        let data =
            Array2::from_shape_vec(
                (sh as usize, sw as usize),
                resized.data_bytes().unwrap().to_vec(),
            )
            .unwrap()
            .mapv(|l| l as f32) / 255.0;

        let (mask, tags) = dauntless::tags2(data);

        let vals = mask.mapv(|b| if b { 255u8 } else { 0u8 });
        let std = vals.as_standard_layout();
        let slice = std.as_slice().unwrap();

        let h = mask.dim().0;

        let mat =
            Mat::from_slice(&slice)
                .unwrap()
                .reshape(1, h as i32)
                .unwrap()
                .clone_pointee();

        let frame = encode(&resize(&resized, 400 / SCALE));
        let mask = encode(&resize(&mat, 400 / SCALE));

        {
            let mut s = state.write().unwrap();
            s.fps = fps;
            s.frame = Some(frame.into());
            s.mask = Some(mask.into());
            s.tags = tags;
        }
    }
}

fn resize(img: &Mat, max: i32) -> Mat {
    let w = img.cols();
    let h = img.rows();

    let mut resized = Mat::default();

    let scale = max as f32 / i32::max(w, h) as f32;
    let sw = (w as f32 * scale) as i32;
    let sh = (h as f32 * scale) as i32;

    imgproc::resize(
        img,
        &mut resized,
        core::Size::new(sw, sh),
        0.0,
        0.0,
        imgproc::INTER_AREA,
    ).unwrap();

    resized
}
