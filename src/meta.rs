use crate::config::Config;

use anyhow::Result;
use nokhwa::Camera;
use nokhwa::pixel_format::LumaFormat;
use nokhwa::utils::{ApiBackend, CameraInfo, FrameFormat, RequestedFormat, RequestedFormatType, Resolution};

use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct Meta {
    pub cams: Vec<(String, Vec<(u32, u32)>)>,
}

impl Meta {
    pub fn new(config: Config) -> Self {
        let mut cams = nokhwa::query(ApiBackend::Auto).unwrap();
        cams.sort_by_key(|c| c.index().as_index().unwrap());

        let info =
            cams
                .iter()
                .filter_map(|c| {
                    let set =
                        (config.server.camera == c.index().as_index().unwrap()).then_some(config.server.res);

                    get_res(c, set)
                        .map(|res| Some((c.human_name(), res)))
                        .unwrap_or(None)
                })
                .collect();

        Self { cams: info }
    }
}

fn get_res(info: &CameraInfo, set: Option<(u32, u32)>) -> Result<Vec<(u32, u32)>> {
    let fmt =
        set
            .map(|(w, h)| RequestedFormatType::HighestResolution(Resolution::new(w, h)))
            .unwrap_or(RequestedFormatType::AbsoluteHighestFrameRate);

    let mut cam = Camera::new(
        info.index().clone(),
        RequestedFormat::new::<LumaFormat>(fmt),
    )?;

    let mut res: Vec<(u32, u32)> =
        cam.compatible_list_by_resolution(FrameFormat::MJPEG)?
            .keys()
            .map(|r| (r.width(), r.height()))
            .collect();

    res.sort();
    Ok(res)
}
