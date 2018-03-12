extern crate journaloo_server;
extern crate rocket;

use journaloo_server::rocket as launch;
use rocket::http::Status;
use rocket::local::Client;

// Note: testing takes rather long
// TODO: properly mock the database

#[test]
fn example() {
    let client = Client::new(launch()).expect("valid rocket instance");
    let response = client.get("/").dispatch();
    assert_eq!(response.status(), Status::Ok);
}
