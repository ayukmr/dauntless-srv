use crate::consts::{CROP, SCALE};
use crate::frame::Frame;

use dauntless::Tag;

use std::io;
use std::io::Write;
use std::sync::Arc;
use std::time::{Instant, SystemTime};

use colored::Colorize;

use arc_swap::ArcSwap;
use ndarray::Array2;

use opencv::{core, imgproc, videoio};
use opencv::core::ToInputArray;
use opencv::prelude::*;

pub struct Data {
    pub fps: Option<f32>,
    pub tags: Vec<Tag>,
    pub frame: Option<Frame>,
    pub mask: Option<Frame>,
    pub time: SystemTime,
}

pub fn update(state: &ArcSwap<Data>) {
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap();

    let fourcc = videoio::VideoWriter::fourcc('M', 'J', 'P', 'G').unwrap();
    let _ = cam.set(videoio::CAP_PROP_FOURCC, fourcc as f64);

    let _ = cam.set(videoio::CAP_PROP_FRAME_WIDTH, 640.0);
    let _ = cam.set(videoio::CAP_PROP_FRAME_HEIGHT, 480.0);
    let _ = cam.set(videoio::CAP_PROP_FPS, 120.0);

    let mut last = Instant::now();
    let mut fps = None;
    let mut tick = 0;

    let mut frame = Mat::default();
    let mut light = Mat::default();

    loop {
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
            print!("\rfps: {}", (fps.unwrap() as i32).to_string().yellow());
            io::stdout().flush().unwrap();
        }
        tick += 1;

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

        let resized = resize(&cropped, 640);

        let sw = resized.cols();
        let sh = resized.rows();

        let data =
            Array2::from_shape_vec(
                (sh as usize, sw as usize),
                resized.data_bytes().unwrap().to_vec(),
            )
            .unwrap()
            .mapv(|l| l as f32 / 255.0);

        let (mask, tags) = dauntless::tags2(&data);

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

        let frame = Frame::encode(&resize(&resized, 640 / SCALE));
        let mask = Frame::encode(&resize(&mat, 640 / SCALE));

        {
            let data = Data {
                fps,
                tags,
                frame: Some(frame),
                mask: Some(mask),
                time: SystemTime::now(),
            };

            state.store(Arc::new(data));
        }
    }
}

fn resize(img: &(MatTraitConst + ToInputArray), max: i32) -> Mat {
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
