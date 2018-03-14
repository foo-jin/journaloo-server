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

use dotenv::dotenv;
use endpoints::*;
use rocket::Rocket;
use std::env;

mod db;
mod endpoints;

pub fn rocket() -> Rocket {
    dotenv().ok();

    // We need to make sure our database_url is set in our `.env` file. This will point to
    // our Postgres database.  If none is supplied, the program will error.
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Initializes database pool with r2d2.
    let pool = db::init_pool(database_url);

    // Configure our server, and mount all routes.  We don't "launch" the server
    // here, but in our `main` procedure.
    rocket::ignite()
        .manage(pool)
        .mount("/", routes![index, user::signup])
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}
