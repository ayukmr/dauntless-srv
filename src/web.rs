use crate::config::Config;
use crate::meta::Meta;
use crate::state::States;

use dauntless::Tag;
use rocket::futures::SinkExt;
use rocket_ws::{Channel, WebSocket};

use std::path::PathBuf;

use colored::Colorize;
use rust_embed::Embed;
use serde_json::json;

use rocket::{Build, Request, Rocket, State as RState};
use rocket::fairing::AdHoc;
use rocket::http::ContentType;
use rocket::serde::json::Json;

pub fn build(states: States) -> Rocket<Build> {
    rocket::build()
        .manage(states)
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
fn data(id: usize, states: &RState<States>, ws: WebSocket) -> Channel<'static> {
    let state = states[id].clone();

    ws.channel(move |mut stream| Box::pin(async move {
        loop {
            state.notify.notified().await;

            let msg = {
                let data = state.data();

                let mut tags: Vec<Tag> = data.tags.iter().map(|t| t.tag).collect();
                tags.sort_by_key(|t| t.id);

                let json = json!({ "ms": data.ms, "tags": tags });
                serde_json::to_string(&json).unwrap()
            };

            stream.send(msg.into()).await?;
        }
    }))
}

#[get("/api/<id>/frame")]
fn frame(id: usize, states: &RState<States>, ws: WebSocket) -> Channel<'static> {
    let state = states[id].clone();

    ws.channel(move |mut stream| Box::pin(async move {
        loop {
            let frame = state.data().frame.clone().unwrap();
            stream.send(frame.into()).await?;

            state.notify.notified().await;
        }
    }))
}

#[get("/api/<id>/mask")]
fn mask(id: usize, states: &RState<States>, ws: WebSocket) -> Channel<'static> {
    let state = states[id].clone();

    ws.channel(move |mut stream| Box::pin(async move {
        loop {
            let mask = state.data().mask.clone().unwrap();
            stream.send(mask.into()).await?;

            state.notify.notified().await;
        }
    }))
}

#[get("/api/<id>/config")]
fn get_config(id: usize, states: &RState<States>) -> Json<Config> {
    let config = states[id].config();
    Json(*config)
}

#[post("/api/<id>/config", data = "<config>")]
fn set_config(id: usize, states: &RState<States>, config: Json<Config>) {
    let mut cfg = config;
    cfg.server.scale = u32::max(cfg.server.scale, 1);

    *states[id].config() = *cfg;

    let configs = states.states.iter().map(|s| *s.config()).collect();
    Config::save_all(configs);
}

#[get("/api/meta")]
fn meta(state: &RState<States>) -> Json<Meta> {
    Json(state.meta.clone())
}
