extern crate journaloo_server;
extern crate rocket;

use rocket::local::Client;
use rocket::http::Status;
use journaloo_server::rocket as launch;

// Note: testing takes rather long
// TODO: properly mock the database

#[test]
fn example() {
    let client = Client::new(launch()).expect("valid rocket instance");
    let mut response = client.get("/").dispatch();
    assert_eq!(response.status(), Status::Ok);
}
