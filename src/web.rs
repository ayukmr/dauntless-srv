use crate::frame::Frame;
use crate::data::St;

use dauntless::{Config, Tag};
use serde::Serialize;

use std::sync::Arc;

use colored::Colorize;

use rocket::{Build, Request, Rocket, State};
use rocket::fs::FileServer;
use rocket::fairing::AdHoc;
use rocket::serde::json::Json;

pub fn build(state: Arc<St>) -> Rocket<Build> {
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
            data,
            frame,
            mask,
            get_config,
            set_config,
            reset_config,
        ])
}

#[catch(404)]
fn not_found(req: &Request<'_>) -> String {
    // println!("\rweb: {}", "404".red());
    format!("no such route: {}", req.uri())
}

#[derive(Serialize)]
struct Sent {
    ms: Option<f32>,
    tags: Vec<Tag>,
}

#[get("/data")]
fn data(state: &State<Arc<St>>) -> Json<Sent> {
    let data = state.data();
    Json(Sent { ms: data.ms, tags: data.tags.clone() })
}

#[get("/frame")]
fn frame(state: &State<Arc<St>>) -> Option<Frame> {
    state.data().frame.clone()
}

#[get("/mask")]
fn mask(state: &State<Arc<St>>) -> Option<Frame> {
    state.data().mask.clone()
}

#[get("/config")]
fn get_config(state: &State<Arc<St>>) -> Json<Config> {
    Json(state.detector().get_config())
}

#[post("/config", data = "<config>")]
fn set_config(state: &State<Arc<St>>, config: Json<Config>) {
    state.detector_wr().set_config(*config);
}

#[post("/config/reset")]
fn reset_config(state: &State<Arc<St>>) -> Json<Config> {
    let config = Config::default();

    state.detector_wr().set_config(config);
    Json(config)
}
