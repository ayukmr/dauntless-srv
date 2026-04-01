use crate::state::State;

use dauntless::{Detector, Tag};
use serde::Serialize;

use std::io::{self, Write};
use std::sync::Arc;
use std::time::{Instant, SystemTime};

use colored::Colorize;

use nokhwa::Camera;
use nokhwa::pixel_format::LumaFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType, Resolution};

pub struct Data {
    pub ms: Option<f32>,
    pub tags: Vec<CameraTag>,
    pub frame: Option<Vec<u8>>,
    pub mask: Option<Vec<u8>>,
    pub time: SystemTime,
}

#[derive(Clone, Copy, Serialize)]
pub struct CameraTag {
    pub camera: u32,
    #[serde(flatten)]
    pub tag: Tag,
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

pub fn update(state: &Arc<State>) {
    let (mut cam_idx, mut w, mut h, mut scale) = {
        let config = state.config();
        (config.server.camera, config.server.res.0, config.server.res.1, config.server.scale)
    };

    let mut camera = create_camera(cam_idx, w, h);
    let mut scale_knl = create_kernel(scale);
    let mut detector = Detector::new();

    let mut tick = 0;

    let mut data = vec![0.0; (w * h) as usize];
    let mut fs = vec![0; (w * h) as usize];
    let mut rsz = vec![0; (w / scale * h / scale) as usize];

    loop {
        (cam_idx, w, h, scale) = {
            let config = state.config();

            let (new_idx, new_w, new_h, new_scale) =
                (config.server.camera, config.server.res.0, config.server.res.1, config.server.scale);

            if (cam_idx, w, h) != (new_idx, new_w, new_h) {
                camera.stop_stream().unwrap();

                if cam_idx != new_idx {
                    camera = create_camera(new_idx, new_w, new_h);
                } else {
                    camera.set_camera_requset(
                        RequestedFormat::new::<LumaFormat>(
                            RequestedFormatType::HighestResolution(
                                Resolution::new(new_w, new_h),
                            ),
                        ),
                    ).unwrap();
                    camera.open_stream().unwrap();
                }

                data = vec![0.0; (new_w * new_h) as usize];
                fs = vec![0; (new_w * new_h) as usize];
                rsz = vec![0; (new_w / new_scale * new_h / new_scale) as usize];
            }
            if scale != new_scale {
                scale_knl = create_kernel(new_scale);
                rsz = vec![0; (new_w / new_scale * new_h / new_scale) as usize];
            }

            (new_idx, new_w, new_h, new_scale)
        };

        let frame = camera.frame().unwrap();
        let decoded = frame.decode_image::<LumaFormat>().unwrap();
        let bytes = decoded.to_vec();

        let start = Instant::now();

        let stride = decoded.len() as u32 / h;
        for y in 0..h {
            let src = (y * stride) as usize;
            let dst = (y * w) as usize;

            for x in 0..w as usize {
                data[dst + x] = bytes[src + x] as f32 / 255.0;
            }
        }

        let (tags, mask) = detector.process(
            w as usize,
            h as usize,
            &state.config().detector,
            &data,
        );

        let now = Instant::now();
        let ms = now.duration_since(start).as_secs_f32() * 1000.0;

        if tick % 10 == 0 {
            print!(
                "\rfps: {} | ms: {}",
                format!("{}", (1000.0 / ms) as i32).to_string().yellow(),
                format!("{:.2}", ms).to_string().yellow(),
            );
            io::stdout().flush().unwrap();

            tick = 0;
        }
        tick += 1;

        encode(w, h, scale, &scale_knl, &data, &mut fs, &mut rsz);
        let fm = rsz.clone();
        encode(w, h, scale, &scale_knl, &mask, &mut fs, &mut rsz);
        let mm = rsz.clone();

        {
            let update = Data {
                tags: tags.iter().map(|t| CameraTag { camera: state.id, tag: *t }).collect(),
                ms: Some(ms),
                frame: Some(fm),
                mask: Some(mm),
                time: SystemTime::now(),
            };

            *state.data() = update;
        }

        state.notify.notify_waiters();
        state.all_notify.notify_waiters();
    }
}

fn create_camera(index: u32, width: u32, height: u32) -> Camera {
    let index = CameraIndex::Index(index);

    let requested = RequestedFormat::new::<LumaFormat>(
        RequestedFormatType::HighestResolution(
            Resolution::new(width, height),
        ),
    );

    let mut camera = Camera::new(index, requested).unwrap();
    camera.open_stream().unwrap();

    camera
}

fn create_kernel(scale: u32) -> Vec<f32> {
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

    scale_knl
}

fn encode<T: Into<f32> + Copy>(
    w: u32,
    h: u32,
    scale: u32,
    scale_knl: &[f32],
    data: &[T],
    fs: &mut [u8],
    rsz: &mut [u8],
) {
    for i in 0..fs.len() {
        fs[i] = (data[i].into() * 255.0) as u8;
    }

    resize(w, h, scale, scale_knl, fs, rsz);
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
