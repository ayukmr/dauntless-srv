#[macro_use] extern crate rocket;
mod consts;
mod frame;
mod data;
mod web;

#[launch]
fn rocket() -> _ {
    web::build()
}
