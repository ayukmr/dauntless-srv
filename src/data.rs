use crate::consts::{CROP, SCALE};
use crate::frame::Frame;

use dauntless::{Detector, Tag};

use std::io;
use std::io::Write;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::{Instant, SystemTime};

use colored::Colorize;

use opencv::{core, imgproc, videoio};
use opencv::core::ToInputArray;
use opencv::prelude::*;

pub struct St {
    pub detector: RwLock<Detector>,
    pub data: RwLock<Data>,
}

pub struct Data {
    pub ms: Option<f32>,
    pub tags: Vec<Tag>,
    pub frame: Option<Frame>,
    pub mask: Option<Frame>,
    pub time: SystemTime,
}

impl St {
    pub fn new(detector: Detector, data: Data) -> Self {
        Self {
            detector: RwLock::new(detector),
            data: RwLock::new(data),
        }
    }

    pub fn detector(&self) -> RwLockReadGuard<'_, Detector> {
        self.detector.read().unwrap()
    }

    pub fn detector_wr(&self) -> RwLockWriteGuard<'_, Detector> {
        self.detector.write().unwrap()
    }

    pub fn data(&self) -> RwLockReadGuard<'_, Data> {
        self.data.read().unwrap()
    }

    pub fn data_wr(&self) -> RwLockWriteGuard<'_, Data> {
        self.data.write().unwrap()
    }
}

pub fn update(state: &Arc<St>) {
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap();

    let fourcc = videoio::VideoWriter::fourcc('M', 'J', 'P', 'G').unwrap();
    let _ = cam.set(videoio::CAP_PROP_FOURCC, fourcc as f64);

    let _ = cam.set(videoio::CAP_PROP_FRAME_WIDTH, 640.0);
    let _ = cam.set(videoio::CAP_PROP_FRAME_HEIGHT, 480.0);
    let _ = cam.set(videoio::CAP_PROP_FPS, 120.0);

    let mut tick = 0;

    let mut frame = Mat::default();
    let mut light = Mat::default();

    let mut mat = None;
    let mut data = None;

    let mut rsz = Mat::default();
    let mut frsz = Mat::default();
    let mut mrsz = Mat::default();

    loop {
        cam.read(&mut frame).unwrap();

        if frame.empty() {
            continue;
        }

        let start = Instant::now();

        imgproc::cvt_color(
            &frame,
            &mut light,
            imgproc::COLOR_BGR2GRAY,
            0,
            core::AlgorithmHint::ALGO_HINT_DEFAULT,
        ).unwrap();

        let w = light.cols();
        let h = light.rows();

        let cw = (w as f32 / CROP) as i32;
        let ch = (h as f32 / CROP) as i32;
        let x = (w - cw) / 2;
        let y = (h - ch) / 2;

        let roi = core::Rect::new(x, y, cw, ch);
        let cropped = Mat::roi(&light, roi).unwrap();

        resize(&cropped, 640, &mut rsz);

        let sw = rsz.cols();
        let sh = rsz.rows();

        let bytes = rsz.data_bytes().unwrap();

        let data = data.get_or_insert_with(|| {
            vec![0.0f32; (sw * sh) as usize]
        });

        for i in 0..data.len() {
            data[i] = bytes[i] as f32 / 255.0;
        }

        let (mask, tags) = state.detector_wr().process(sw as usize, sh as usize, &data);

        let now = Instant::now();
        let ms = now.duration_since(start).as_secs_f32() * 1000.0;

        if tick % 10 == 0 {
            print!(
                "\rms: {} | fps: {}",
                format!("{:.2}", ms).to_string().yellow(),
                format!("{}", (1000.0 / ms) as i32).to_string().yellow(),
            );
            io::stdout().flush().unwrap();
        }
        tick += 1;

        let mat = mat.get_or_insert_with(|| {
            Mat::zeros(sh, sw, core::CV_8UC1).unwrap().to_mat().unwrap()
        });
        let mts = mat.data_bytes_mut().unwrap();

        for i in 0..mask.len() {
            mts[i] = mask[i] * 255;
        }

        resize(&rsz, 640 / SCALE, &mut frsz);
        resize(mat, 640 / SCALE, &mut mrsz);

        let frame = Frame::encode(&frsz);
        let mask = Frame::encode(&mrsz);

        {
            let update = Data {
                tags,
                ms: Some(ms),
                frame: Some(frame),
                mask: Some(mask),
                time: SystemTime::now(),
            };

            *state.data_wr() = update;
        }
    }
}

fn resize<T: MatTraitConst + ToInputArray>(img: &T, max: i32, out: &mut Mat)  {
    let w = img.cols();
    let h = img.rows();

    let scale = max as f32 / i32::max(w, h) as f32;
    let sw = (w as f32 * scale) as i32;
    let sh = (h as f32 * scale) as i32;

    imgproc::resize(
        img,
        out,
        core::Size::new(sw, sh),
        0.0,
        0.0,
        imgproc::INTER_LINEAR,
    ).unwrap();
}
