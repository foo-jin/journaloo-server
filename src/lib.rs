#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(custom_attribute)]
#![feature(custom_derive)]

extern crate bcrypt;
extern crate chrono;
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate jsonwebtoken as jwt;
#[cfg(test)]
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate pretty_env_logger as env_logger;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use db::init_pool;
use endpoints::*;
use rocket::Rocket;

pub mod db;
mod endpoints;

pub fn rocket() -> Rocket {
    dotenv::dotenv().ok();

    env_logger::init();
    let pool = init_pool();

    // Configure our server, and mount all routes.  We don't "launch" the server
    // here, but in our `main` procedure.
    rocket::ignite().manage(pool).mount(
        "/",
        routes![index, user::signup, user::update, user::delete, user::login],
    )
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}
