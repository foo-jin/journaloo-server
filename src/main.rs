#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate diesel;
extern crate dotenv;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rocket;
extern crate rocket_contrib;

mod db;
mod endpoints;

use std::env;

use rocket::Rocket;
use dotenv::dotenv;


fn rocket() -> Rocket {
    dotenv().ok();

    // We need to make sure our database_url is set in our `.env` file. This will point to
    // our Postgres database.  If none is supplied, the program will error.
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Initializes database pool with r2d2.
    let pool = db::init_pool(database_url);

    // Configure our server, and mount all routes.  We don't "launch" the server
    // here, but in our `main` procedure.
    rocket::ignite().manage(pool).mount(
        "/",
        routes![
            index,
        ],
    )
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

fn main() {
    rocket().launch();
}