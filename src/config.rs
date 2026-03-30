use dauntless::Config as DetectorConfig;

use std::env;
use std::io::{BufReader, BufWriter};
use std::fs::File;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use nokhwa::Camera;
use nokhwa::pixel_format::LumaFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Config {
    pub detector: DetectorConfig,
    pub server: ServerConfig,
}

impl Config {
    pub fn default(index: u32) -> Self {
        Self {
            detector: DetectorConfig::default(),
            server: ServerConfig::default(index),
        }
    }

    pub fn load_all() -> Result<Vec<Self>> {
        let file = File::open(path())?;
        let configs: Vec<Config> = serde_json::from_reader(BufReader::new(file))?;
        Ok(configs)
    }

    pub fn save_all(configs: Vec<Self>) {
        let file = File::create(path()).unwrap();
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &configs).unwrap();
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ServerConfig {
    pub camera: u32,
    pub res: (u32, u32),
    pub scale: u32,
}

impl ServerConfig {
    fn default(index: u32) -> Self {
        let cam = Camera::new(
            CameraIndex::Index(index),
            RequestedFormat::new::<LumaFormat>(RequestedFormatType::AbsoluteHighestFrameRate),
        ).unwrap();
        let res = cam.resolution();

        Self {
            scale: 1,
            camera: cam.index().as_index().unwrap(),
            res: (res.width(), res.height()),
        }
    }
}

fn path() -> PathBuf {
    env::current_exe().unwrap().parent().unwrap().join("dauntless.json")
}
