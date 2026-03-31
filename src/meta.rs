use std::collections::HashMap;

use crate::config::Config;

use anyhow::Result;
use nokhwa::Camera;
use nokhwa::pixel_format::LumaFormat;
use nokhwa::utils::{ApiBackend, CameraInfo, FrameFormat, RequestedFormat, RequestedFormatType, Resolution};

use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct Meta {
    n_cams: u32,
    cams: HashMap<u32, (String, Vec<(u32, u32)>)>,
}

impl Meta {
    pub fn new(n_cams: u32, configs: &[Config]) -> Self {
        let mut cams = nokhwa::query(ApiBackend::Auto).unwrap();
        cams.sort_by_key(|c| c.index().as_index().unwrap());

        let cam_res: HashMap<u32, (u32, u32)> =
            configs
                .iter()
                .map(|c| (c.server.camera, c.server.res))
                .collect();

        let info =
            cams
                .iter()
                .filter_map(|c| {
                    let idx = c.index().as_index().unwrap();
                    let set = cam_res.get(&idx).copied();

                    get_res(c, set)
                        .map(|res| Some((idx, (c.human_name(), res))))
                        .unwrap_or(None)
                })
                .collect();

        Self { n_cams, cams: info }
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
