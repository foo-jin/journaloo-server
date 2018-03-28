#![allow(non_upper_case_globals)]

extern crate dotenv;
extern crate journaloo_server;
extern crate jsonwebtoken as jwt;
#[macro_use]
extern crate lazy_static;
extern crate rocket;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::env;

use journaloo_server::rocket as launch;

use rocket::http::{ContentType, Status};
use rocket::local::Client;

#[derive(Serialize, Debug)]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub email: &'a str,
    pub password: &'a str,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
    pub email: String,
}

#[derive(Serialize, Debug)]
pub struct UserLogin<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

lazy_static! {
    static ref JD_INFO: UserInfo = UserInfo {
        id: 101,
        username: "JonDoe2".to_string(),
        email: "jon2@doe.com".to_string(),
    };

    static ref JD_LOGIN: UserLogin<'static> = UserLogin {
        username: &JD_INFO.username,
        password: "asdf",
    };

    static ref SECRET: String = {
        dotenv::dotenv().ok();
        env::var("JWT_SECRET").expect("SECRET must be set")
    };

    static ref client: Client = Client::new(launch()).expect("valid rocket instance");
}

fn check_signup(user: &NewUser) {
    let response = client
        .post("/user")
        .header(ContentType::JSON)
        .body(serde_json::to_string(user).expect("failed to serialize"))
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
fn signup_errors() {
    let mut user = NewUser {
        username: "jondoe",
        email: "notjon@doe.com",
        password: "asdf",
    };

    check_signup(&user);

    user.username = "notjondoe";
    user.email = "jon@doe.com";

    check_signup(&user);
}

#[test]
fn get() {
    let mut response = client.get(format!("/user/{}", JD_INFO.id)).dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let result: UserInfo = serde_json::from_str(&response.body_string().expect("no body found"))
        .expect("failed to deserialize");

    assert_eq!(result, *JD_INFO);

    response = client.get(format!("user/{}", JD_INFO.id - 1)).dispatch();

    assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn login() {
    let login = serde_json::to_string(&*JD_LOGIN).expect("failed to serialize");
    let mut response = client
        .post("/user/login")
        .header(ContentType::JSON)
        .body(login)
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::Plain));

    let body = response.body_string().expect("no body found");
    let token = jwt::decode::<UserInfo>(&body, SECRET.as_bytes(), &jwt::Validation::default())
        .expect("failed to decode auth token");

    assert_eq!(token.claims, *JD_INFO);

    let login = UserLogin {
        username: "jondoe",
        password: "aaaa",
    };

    response = client
        .post("/user/login")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&login).expect("failed to serialize"))
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized)
}

#[test]
fn reset_password_errors() {
    let response = client.put("/user/asdf***@madeup.com/reset").dispatch();

    assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn paging_errors() {
    let mut response = client.get("/user/all?page=0").dispatch();
    let users: Vec<UserInfo> = serde_json::from_str(&response.body_string().expect("no body found")).expect("failed to deserialize");
    users.iter().for_each(|u| println!("{:?}", u));

    response = client.get("/user?page=-1").dispatch();
    assert_eq!(response.status(), Status::NotFound)
}
