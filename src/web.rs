use crate::consts::CROP;
use crate::frame::Frame;
use crate::data::Data;

use dauntless::{Config, Tag};

use std::sync::Arc;
use arc_swap::ArcSwap;

use colored::Colorize;

use rocket::{Build, Request, Rocket, State};
use rocket::fs::FileServer;
use rocket::fairing::AdHoc;
use rocket::serde::json::Json;

const FOV: f32 = 48_f32;

pub fn build(state: Arc<ArcSwap<Data>>) -> Rocket<Build> {
    let fov = 2.0 * ((FOV.to_radians() * 0.5).tan() / CROP).atan();

    dauntless::set_config(Config {
        fov_rad: fov,
        harris_k: 0.04,
        harris_thresh: 0.001,
        hyst_low: 0.1,
        hyst_high: 0.2,
        ..Config::default()
    });

    rocket::build()
        .manage(state)
        .attach(AdHoc::on_liftoff(
            "log",
            |_| Box::pin(async move {
                println!("web: {}", "connected".green());
            })
        ))
        .register("/", catchers![not_found])
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

#[catch(404)]
fn not_found(req: &Request<'_>) -> String {
    println!("\rweb: {}", "404".red());
    format!("no such route: {}", req.uri())
}

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
