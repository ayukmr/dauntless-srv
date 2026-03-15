use crate::consts::{self, CAMERA, HEIGHT, SCALE, WIDTH};
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
    pub edges: Option<Frame>,
    pub corners: Option<Frame>,
    pub time: SystemTime,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            ms: None,
            tags: Vec::new(),
            frame: None,
            edges: None,
            corners: None,
            time: SystemTime::now(),
        }
    }
}

impl St {
    pub fn new(detector: Detector) -> Self {
        Self {
            detector: RwLock::new(detector),
            data: RwLock::new(Data::default()),
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
    let mut cam = videoio::VideoCapture::new(
        *CAMERA.get_or_init(|| consts::cfg().camera),
        videoio::CAP_ANY,
    ).unwrap();

    let fourcc = videoio::VideoWriter::fourcc('M', 'J', 'P', 'G').unwrap();
    let _ = cam.set(videoio::CAP_PROP_FOURCC, fourcc as f64);

    let _ = cam.set(
        videoio::CAP_PROP_FRAME_WIDTH,
        *WIDTH.get_or_init(|| consts::cfg().width),
    );
    let _ = cam.set(
        videoio::CAP_PROP_FRAME_HEIGHT,
        *HEIGHT.get_or_init(|| consts::cfg().height),
    );
    let _ = cam.set(videoio::CAP_PROP_FPS, 120.0);

    let mut tick = 0;

    let mut frame = Mat::default();
    let mut light = Mat::default();

    let mut mat = None;
    let mut data = None;

    let mut rsz = Mat::default();

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

        let bytes = light.data_bytes().unwrap();

        let data = data.get_or_insert_with(|| {
            vec![0.0f32; (w * h) as usize]
        });

        for i in 0..data.len() {
            data[i] = bytes[i] as f32 / 255.0;
        }

        let (tags, edges, corners) = state.detector_wr().process(w as usize, h as usize, data);

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

        let scale = *SCALE.get_or_init(|| consts::cfg().scale);
        resize(&light, (w as f32 / scale as f32) as u32, &mut rsz);

        let fm = Frame::encode(&rsz);
        let em = encode(w, h, scale, edges, &mut mat, &mut rsz);
        let cm = encode(w, h, scale, corners, &mut mat, &mut rsz);

        {
            let update = Data {
                tags,
                ms: Some(ms),
                frame: Some(fm),
                edges: Some(em),
                corners: Some(cm),
                time: SystemTime::now(),
            };

            *state.data_wr() = update;
        }
    }
}

fn resize<T: MatTraitConst + ToInputArray>(img: &T, max: u32, out: &mut Mat)  {
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

fn encode(w: i32, h: i32, scale: u32, mask: Vec<u8>, mat: &mut Option<Mat>, rsz: &mut Mat) -> Frame {
    let mat = mat.get_or_insert_with(|| {
        Mat::zeros(h, w, core::CV_8UC1).unwrap().to_mat().unwrap()
    });
    let mts = mat.data_bytes_mut().unwrap();

    for i in 0..mask.len() {
        mts[i] = mask[i] * 255;
    }

    resize(mat, (w as f32 / scale as f32) as u32, rsz);
    Frame::encode(&rsz)
}
