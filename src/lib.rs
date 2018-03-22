#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(custom_attribute)]
#![feature(custom_derive)]
#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]

extern crate bcrypt;
extern crate chrono;
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate jsonwebtoken as jwt;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate pretty_env_logger as env_logger;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rand;
extern crate rocket;
extern crate rocket_contrib;
extern crate sendgrid;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use db::init_pool;
use endpoints::{entry, user};
use rocket::Rocket;
use std::env;

mod db;
mod endpoints;

lazy_static!(
    static ref SECRET: String = env::var("JWT_SECRET").expect("SECRET must be set");
);

pub fn rocket() -> Rocket {
    dotenv::dotenv().ok();

    //let _ = env_logger::try_init();
    let pool = init_pool();

    // Configure our server, and mount all routes.  We don't "launch" the server
    // here, but in our `main` procedure.
    rocket::ignite().manage(pool).mount(
        "/",
        routes![
            index,
            user::signup,
            user::update,
            user::delete,
            user::login,
            user::get_by_id,
            user::reset_password,
            entry::create,
            entry::delete,
            entry::get_all,
            entry::get_by_journey
        ],
    )
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}