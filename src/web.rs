use crate::consts::CROP;
use crate::frame::Frame;
use crate::data::Data;

use dauntless::{Config, Tag};

use std::sync::Arc;
use arc_swap::ArcSwap;

use rocket::{Build, Rocket, State};
use rocket::fs::FileServer;
use rocket::serde::json::Json;

#[get("/fps")]
fn fps(state: &State<Arc<ArcSwap<Data>>>) -> Json<Option<f32>> {
    Json(state.load().fps)
}

#[get("/tags")]
fn tags(state: &State<Arc<ArcSwap<Data>>>) -> Json<Vec<Tag>> {
    Json(state.load().tags.clone())
}

#[get("/frame")]
fn frame(state: &State<Arc<ArcSwap<Data>>>) -> Option<Frame> {
    state.load().frame.clone()
}

#[get("/mask")]
fn mask(state: &State<Arc<ArcSwap<Data>>>) -> Option<Frame> {
    state.load().mask.clone()
}

#[get("/config")]
fn get_config() -> Json<Config> {
    Json(dauntless::get_config())
}

#[post("/config", data = "<config>")]
fn set_config(config: Json<Config>) {
    dauntless::set_config(*config);
}

#[post("/config/reset")]
fn reset_config() -> Json<Config> {
    let config = Config::default();

    dauntless::set_config(config);
    Json(config)
}

pub fn build(state: Arc<ArcSwap<Data>>) -> Rocket<Build> {
    dauntless::set_config(Config {
        fov_rad: 72_f32.to_radians() / CROP,
        harris_k: 0.04,
        harris_thresh: 0.001,
        hyst_low: 0.1,
        hyst_high: 0.2,
        filter_enclosed: false,
        ..Config::default()
    });

    rocket::build()
        .manage(state)
        .mount("/", FileServer::from("./www"))
        .mount("/api", routes![
            fps,
            tags,
            frame,
            mask,
            get_config,
            set_config,
            reset_config,
        ])
}
