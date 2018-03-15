extern crate journaloo_server;
extern crate jsonwebtoken as jwt;
#[macro_use]
extern crate lazy_static;
extern crate rocket;
extern crate serde;
extern crate serde_json;

use journaloo_server::rocket as launch;
use journaloo_server::db::models::user::UserInfo;
use rocket::http::{ContentType, Status};
use rocket::local::Client;
use serde_json::from_str;

lazy_static! {
    static ref JONDOE: UserInfo = UserInfo {
        id: 19,
        username: "jondoe".to_string(),
        email: "jon@doe.com".to_string(),
    };
}

#[test]
#[ignore]
fn get() {
    let client = Client::new(launch()).expect("valid rocket instance");
    let mut response = client.get(format!("/user/{}", JONDOE.id)).dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let result: UserInfo =
        from_str(&response.body_string().expect("no body found")).expect("failed to deserialize");
    assert_eq!(result, *JONDOE)
}
