#[macro_use] extern crate rocket;

use dauntless::{Config, Tag};

use std::io;
use std::io::Write;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use std::thread;

use rocket::{Request, Response, State};
use rocket::fs::FileServer;
use rocket::serde::json::Json;
use rocket::http::ContentType;
use rocket::response;
use rocket::response::Responder;

use ndarray::Array2;

use opencv::{core, imgproc, videoio};
use opencv::core::ToInputArray;
use opencv::prelude::*;

const SCALE: i32 = 4;

#[derive(Clone)]
struct Frame {
    pub width: i32,
    pub height: i32,
    pub data: Arc<[u8]>,
}

impl Frame {
    fn encode(mat: &Mat) -> Self {
        let width = mat.cols();
        let height = mat.rows();

        let bytes = mat.data_bytes().unwrap().to_vec();

        Self {
            width,
            height,
            data: bytes.into(),
        }
    }
}

impl<'r> Responder<'r, 'static> for Frame {
    fn respond_to(self, _req: &'r Request<'_>) -> response::Result<'static> {
        Response::build()
            .header(ContentType::Binary)
            .raw_header("X-Width", self.width.to_string())
            .raw_header("X-Height", self.height.to_string())
            .raw_header("X-Scale", SCALE.to_string())
            .sized_body(self.data.len(), io::Cursor::new(Arc::clone(&self.data)))
            .ok()
    }
}

struct Data {
    pub fps: Option<f32>,
    pub tags: Vec<Tag>,
    pub frame: Option<Frame>,
    pub mask: Option<Frame>,
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
fn frame(state: &State<Arc<RwLock<Data>>>) -> Option<Frame> {
    Some(state.read().ok()?.frame.as_ref()?.clone())
}

#[get("/mask")]
fn mask(state: &State<Arc<RwLock<Data>>>) -> Option<Frame> {
    Some(state.read().ok()?.mask.as_ref()?.clone())
}

#[get("/config")]
fn get_config() -> Json<Config> {
    Json(dauntless::get_config())
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

        let resized = resize(&cropped, 400);

        let sw = resized.cols();
        let sh = resized.rows();

        let data =
            Array2::from_shape_vec(
                (sh as usize, sw as usize),
                resized.data_bytes().unwrap().to_vec(),
            )
            .unwrap()
            .mapv(|l| l as f32 / 255.0);

        let (mask, tags) = dauntless::tags2(data);

        let vals = mask.mapv(|b| if b { 255u8 } else { 0u8 });
        let std = vals.as_standard_layout();
        let slice = std.as_slice().unwrap();

        let h = mask.dim().0;

        let mat =
            Mat::from_slice(slice)
                .unwrap()
                .reshape(1, h as i32)
                .unwrap()
                .clone_pointee();

        let frame = Frame::encode(&resize(&resized, 400 / SCALE));
        let mask = Frame::encode(&resize(&mat, 400 / SCALE));

        {
            let mut s = state.write().unwrap();
            s.fps = fps;
            s.frame = Some(frame.into());
            s.mask = Some(mask.into());
            s.tags = tags;
        }
    }
}

fn resize<T>(img: &T, max: i32) -> Mat
where
    T: MatTraitConst + ToInputArray,
{
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
