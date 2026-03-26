use nokhwa::Camera;
use nokhwa::pixel_format::LumaFormat;
use nokhwa::utils::{ApiBackend, CameraInfo, FrameFormat, RequestedFormat, RequestedFormatType};

use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct Meta {
    pub cams: Vec<(String, Vec<(u32, u32)>)>,
}

impl Meta {
    pub fn new() -> Self {
        let mut cams = nokhwa::query(ApiBackend::Auto).unwrap();
        cams.sort_by_key(|c| c.index().as_index().unwrap());

        let info = cams.iter().map(|c| (c.human_name(), get_res(c))).collect();
        Self { cams: info }
    }
}

fn get_res(info: &CameraInfo) -> Vec<(u32, u32)> {
    let mut cam = Camera::new(
        info.index().clone(),
        RequestedFormat::new::<LumaFormat>(RequestedFormatType::AbsoluteHighestFrameRate),
    ).unwrap();

    let mut res: Vec<(u32, u32)> =
        cam.compatible_list_by_resolution(FrameFormat::MJPEG)
            .unwrap()
            .iter()
            .map(|(r, _)| (r.width(), r.height()))
            .collect();

    res.sort();
    res
}
