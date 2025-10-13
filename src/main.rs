#[macro_use] extern crate rocket;

use dauntless::{Filter, Tag};

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
}

#[get("/tags")]
fn tags(data: &State<Arc<RwLock<Data>>>) -> Json<Vec<Tag>> {
    Json(data.read().unwrap().tags.clone())
}

#[get("/frame")]
fn frame(data: &State<Arc<RwLock<Data>>>) -> Json<Vec<Vec<f32>>> {
    Json(data.read().unwrap().frame.clone())
}

#[launch]
fn rocket() -> _ {
    let data = Arc::new(
        RwLock::new(
            Data {
                tags: Vec::new(),
                frame: Vec::new(),
            },
        ),
    );

    let data_cln = Arc::clone(&data);

    thread::spawn(move || {
        let mut cam = videoio::VideoCapture::new(2, videoio::CAP_ANY).unwrap();

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

            {
                let mut d = data_cln.write().unwrap();

                d.frame =
                    data.rows()
                        .into_iter()
                        .map(|row| row.to_vec())
                        .collect();

                d.tags = dauntless::tags_custom(
                    data,
                    Filter { quads: true, paras: true, enclosed: false },
                );
            }
        }
    });

    rocket::build()
        .manage(data)
        .mount("/", FileServer::from("./www"))
        .mount("/api", routes![tags, frame])
}
