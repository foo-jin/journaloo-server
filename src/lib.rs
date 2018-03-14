#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(custom_attribute)]

extern crate bcrypt;
extern crate chrono;
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate jsonwebtoken as jwt;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use endpoints::*;
use rocket::Rocket;
use db::init_pool;

mod db;
mod endpoints;

pub fn rocket() -> Rocket {
    let pool = init_pool();

    // Configure our server, and mount all routes.  We don't "launch" the server
    // here, but in our `main` procedure.
    rocket::ignite()
        .manage(pool)
        .mount("/", routes![index, certbot, user::signup])
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/.well-known/acme-challenge")]
fn certbot() -> String {
    ::std::env::var("LETS_ENCRYPT_CHALLENGE").unwrap_or_else(|_| "not set".to_string())
}
