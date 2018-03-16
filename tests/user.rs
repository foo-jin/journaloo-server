extern crate journaloo_server;
extern crate jsonwebtoken as jwt;
#[macro_use]
extern crate lazy_static;
extern crate rocket;
extern crate serde;
extern crate serde_json;

use journaloo_server::rocket as launch;
use journaloo_server::db::models::user::{UserInfo};
use journaloo_server::endpoints::user::UserLogin;
use rocket::http::{ContentType, Status};
use rocket::local::Client;
use serde_json::from_str;

lazy_static! {
    static ref JD_INFO: UserInfo = UserInfo {
        id: 19,
        username: "jondoe".to_string(),
        email: "jon@doe.com".to_string(),
    };

    static ref JD_LOGIN: UserLogin = UserLogin {
        username: JD_INFO.username.clone(),
        password: "asdf".to_string(),
    };
}

#[test]
fn get() {
    let client = Client::new(launch()).expect("valid rocket instance");
    let mut response = client.get(format!("/user/{}", JD_INFO.id)).dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let result: UserInfo =
        from_str(&response.body_string().expect("no body found")).expect("failed to deserialize");
    assert_eq!(result, *JD_INFO)
}

#[test]
fn login() {
    let client = Client::new(launch()).expect("valid rocket instance");
    let login = serde_json::to_string(&*JD_LOGIN).expect("failed to serialize");
    let mut response = client
        .post("/user/login")
        .header(ContentType::JSON)
        .body(login)
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::Plain));

    let token = jwt::decode::<UserInfo>(
        &response.body_string().expect("no body found"),
        b"secret",
        &jwt::Validation::default(),
    ).expect("failed to decode auth token");

    assert_eq!(token.claims, *JD_INFO);
}
