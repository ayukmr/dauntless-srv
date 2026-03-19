use crate::consts::{self, CAMERA, HEIGHT, SCALE, WIDTH};
use crate::frame::Frame;

use dauntless::{Detector, Tag};
use nokhwa::Camera;
use nokhwa::pixel_format::LumaFormat;
use nokhwa::utils as nutils;

use std::io::{self, Write};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::{Instant, SystemTime};

use colored::Colorize;

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

impl Default for Data {
    fn default() -> Self {
        Self {
            ms: None,
            tags: Vec::new(),
            frame: None,
            mask: None,
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
    let index = nutils::CameraIndex::Index(*CAMERA.get_or_init(|| consts::cfg().camera));

    let requested = nutils::RequestedFormat::new::<LumaFormat>(
        nutils::RequestedFormatType::Closest(
            nutils::CameraFormat::new(
                nutils::Resolution::new(
                    *WIDTH.get_or_init(|| consts::cfg().width),
                    *HEIGHT.get_or_init(|| consts::cfg().height),
                ),
                nutils::FrameFormat::YUYV,
                1000,
            ),
        ),
    );

    let mut camera = Camera::new(index, requested).unwrap();
    camera.open_stream().unwrap();

    let mut tick = 0;

    let mut data = None;
    let mut fs = None;
    let mut rsz = None;

    let scale = *SCALE.get_or_init(|| consts::cfg().scale);

    let sigma = scale as f32 / 3.0;
    let two_sigma_sq = 2.0 * sigma*sigma;
    let center = (scale as f32 - 1.0) / 2.0;

    let mut scale_knl: Vec<f32> =
        (0..scale).flat_map(|y|
            (0..scale).map(move |x| {
                let xd = x as f32 - center;
                let yd = y as f32 - center;
                let d = xd*xd + yd*yd;

                (-d / two_sigma_sq).exp()
            })
        ).collect();

    let sum: f32 = scale_knl.iter().sum();
    for v in &mut scale_knl {
        *v /= sum;
    }

    loop {
        let frame = camera.frame().unwrap();
        let decoded = frame.decode_image::<LumaFormat>().unwrap();

        let (w, h) = decoded.dimensions();
        let bytes = decoded.to_vec();

        let start = Instant::now();

        let data = data.get_or_insert_with(|| vec![0.0; (w * h) as usize]);

        for i in 0..data.len() {
            data[i] = bytes[i] as f32 / 255.0;
        }

        let (tags, mask) = state.detector_wr().process(w as usize, h as usize, data);

        let now = Instant::now();
        let ms = now.duration_since(start).as_secs_f32() * 1000.0;

        if tick % 10 == 0 {
            print!(
                "\rfps: {} | ms: {}",
                format!("{}", (1000.0 / ms) as i32).to_string().yellow(),
                format!("{:.2}", ms).to_string().yellow(),
            );
            io::stdout().flush().unwrap();
        }
        tick += 1;

        let sw = w / scale;
        let sh = h / scale;

        let fs = fs.get_or_insert_with(|| vec![0; (w * h) as usize]);
        let rsz = rsz.get_or_insert_with(|| vec![0; (sw * sh) as usize]);

        resize(w, h, scale, &scale_knl, &bytes, rsz);
        let fm = Frame::encode(sw, sh, rsz);
        let mm = encode(w, h, scale, &scale_knl, &mask, fs, rsz);

        {
            let update = Data {
                tags,
                ms: Some(ms),
                frame: Some(fm),
                mask: Some(mm),
                time: SystemTime::now(),
            };

            *state.data_wr() = update;
        }
    }
}

fn resize(w: u32, h: u32, scale: u32, scale_knl: &[f32], img: &[u8], out: &mut [u8])  {
    if scale == 1 {
        out.copy_from_slice(img);
        return;
    }

    let sw = w / scale;
    let sh = h / scale;

    for y in 0..sh {
        let oy = y * scale;
        let r = y * sw;

        for x in 0..sw {
            let ox = x * scale;
            let i = r + x;

            let mut sum = 0.0;

            for ky in 0..scale {
                let yy = oy + ky;
                let rr = yy * w;
                let kr = ky * scale;

                for kx in 0..scale {
                    let xx = ox + kx;
                    let ii = rr + xx;
                    let ki = kr + kx;

                    let v = img[ii as usize] as f32;
                    let kv = scale_knl[ki as usize];

                    sum += v * kv;
                }
            }

            out[i as usize] = sum as u8;
        }
    }
}

fn encode(w: u32, h: u32, scale: u32, scale_knl: &[f32], mask: &[u8], fs: &mut [u8], rsz: &mut [u8]) -> Frame {
    for i in 0..mask.len() {
        fs[i] = mask[i] * 255;
    }

    resize(w, h, scale, scale_knl, fs, rsz);
    Frame::encode(w / scale, h / scale, rsz)
}
