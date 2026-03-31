use crate::config::Config;
use crate::data::St;
use crate::frame::Frame;
use crate::meta::Meta;

use dauntless::Tag;

use std::path::PathBuf;
use std::sync::Arc;

use colored::Colorize;
use rust_embed::Embed;
use serde_json::{Value, json};

use rocket::{Build, Request, Rocket, State};
use rocket::fairing::AdHoc;
use rocket::http::ContentType;
use rocket::serde::json::Json;

pub fn build(meta: Meta, states: Vec<Arc<St>>) -> Rocket<Build> {
    rocket::build()
        .manage(states)
        .manage(meta)
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
            meta,
            frame,
            mask,
            get_config,
            set_config,
        ])
}

#[catch(404)]
fn not_found(req: &Request<'_>) -> String {
    println!("\rweb: {}", "404".red());
    format!("error with route: {}", req.uri())
}

#[derive(Embed)]
#[folder = "www/dist"]
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
            path.split('.').next_back().unwrap_or("")
        ).unwrap_or(ContentType::Binary);

        (content_type, data.data.into_owned())
    })
}

#[get("/api/<id>/data")]
fn data(id: usize, states: &State<Vec<Arc<St>>>) -> Json<Value> {
    let state = &states.inner()[id];
    let data = state.data();

    let mut tags: Vec<Tag> = data.tags.iter().map(|t| t.tag).collect();
    tags.sort_by_key(|t| t.id);

    Json(json!({ "ms": data.ms, "tags": tags }))
}

#[get("/api/<id>/frame")]
fn frame(id: usize, states: &State<Vec<Arc<St>>>) -> Option<Frame> {
    let state = &states.inner()[id];
    state.data().frame.clone()
}

#[get("/api/<id>/mask")]
fn mask(id: usize, states: &State<Vec<Arc<St>>>) -> Option<Frame> {
    let state = &states.inner()[id];
    state.data().mask.clone()
}

#[get("/api/<id>/config")]
fn get_config(id: usize, states: &State<Vec<Arc<St>>>) -> Json<Config> {
    let state = &states.inner()[id];
    Json(*state.config())
}

#[post("/api/<id>/config", data = "<config>")]
fn set_config(id: usize, states: &State<Vec<Arc<St>>>, config: Json<Config>) {
    let states = states.inner();
    let state = &states[id];

    let mut cfg = config;
    cfg.server.scale = u32::max(cfg.server.scale, 1);

    *state.config_wr() = *cfg;

    let configs = states.iter().map(|s| *s.config()).collect();
    Config::save_all(configs);
}

#[get("/api/meta")]
fn meta(state: &State<Meta>) -> Json<Meta> {
    Json(state.inner().clone())
}
