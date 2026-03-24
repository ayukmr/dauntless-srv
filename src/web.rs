use crate::frame::Frame;
use crate::data::St;

use dauntless::Config;

use std::path::PathBuf;
use std::sync::Arc;

use colored::Colorize;
use rust_embed::Embed;
use serde_json::{Value, json};

use rocket::{Build, Request, Rocket, State};
use rocket::fairing::AdHoc;
use rocket::http::ContentType;
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
        .mount("/", routes![
            index,
            files,
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
    println!("\rweb: {}", "404".red());
    format!("no such route: {}", req.uri())
}

#[derive(Embed)]
#[folder = "www"]
struct Files;

#[get("/")]
fn index() -> (ContentType, Vec<u8>) {
    (
        ContentType::HTML,
        Files::get("index.html").unwrap().data.into_owned(),
    )
}

#[get("/<file..>")]
fn files(file: PathBuf) -> Option<(ContentType, Vec<u8>)> {
    let path = file.to_string_lossy();

    Files::get(path.as_ref()).map(|data| {
        let content_type = ContentType::from_extension(
            path.split('.').last().unwrap_or("")
        ).unwrap_or(ContentType::Binary);

        (content_type, data.data.into_owned())
    })
}

#[get("/api/data")]
fn data(state: &State<Arc<St>>) -> Json<Value> {
    let data = state.data();
    Json(json!({ "ms": data.ms, "tags": data.tags.clone() }))
}

#[get("/api/frame")]
fn frame(state: &State<Arc<St>>) -> Option<Frame> {
    state.data().frame.clone()
}

#[get("/api/mask")]
fn mask(state: &State<Arc<St>>) -> Option<Frame> {
    state.data().mask.clone()
}

#[get("/api/config")]
fn get_config(state: &State<Arc<St>>) -> Json<Config> {
    Json(state.detector().get_config())
}

#[post("/api/config", data = "<config>")]
fn set_config(state: &State<Arc<St>>, config: Json<Config>) {
    state.detector_wr().set_config(*config);
}

#[post("/api/config/reset")]
fn reset_config(state: &State<Arc<St>>) -> Json<Config> {
    let config = Config::default();

    state.detector_wr().set_config(config);
    Json(config)
}
