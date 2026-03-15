use std::cell::OnceCell;

use std::env;
use std::fs::File;
use std::io::BufReader;

use serde::Deserialize;

pub const SCALE: OnceCell<u32> = OnceCell::new();
pub const CAMERA: OnceCell<i32> = OnceCell::new();
pub const WIDTH: OnceCell<f64> = OnceCell::new();
pub const HEIGHT: OnceCell<f64> = OnceCell::new();

#[derive(Deserialize)]
pub struct Config {
    pub scale: u32,
    pub camera: i32,
    pub width: f64,
    pub height: f64,
}

pub fn cfg() -> Config {
    let path = env::current_exe().unwrap().parent().unwrap().join("dauntless.json");
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    serde_json::from_reader(reader).unwrap()
}
