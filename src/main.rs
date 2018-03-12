extern crate journaloo_server;
extern crate rocket;

use journaloo_server::rocket;
use rocket::Rocket;

fn main() {
    rocket().launch();
}
