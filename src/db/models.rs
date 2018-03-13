use chrono::NaiveDateTime;
use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use jwt::{decode, Validation};
use rocket::http::Status;
use rocket::Outcome;
use rocket::request::{self, FromRequest};
use rocket::Request;
use super::schema::*;

#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub date: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
#[table_name = "users"]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub date: NaiveDateTime,
}

#[derive(Queryable)]
pub struct Journey {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub archived: bool,
    pub start_date: NaiveDateTime,
    pub end_date: Option<NaiveDateTime>,
}

#[derive(Insertable)]
#[table_name = "journeys"]
pub struct NewJourney<'a> {
    pub user_id: i32,
    pub title: &'a str,
}

#[derive(Queryable)]
#[table_name = "entries"]
pub struct Entry {
    pub id: i32,
    pub journey_id: i32,
    pub created: NaiveDateTime,
    pub archived: bool,
    pub description: Option<String>,
    pub coordinates: Option<String>,
    pub location: Option<String>,
}

#[derive(Insertable)]
#[table_name = "entries"]
pub struct NewEntry<'a> {
    pub journey_id: i32,
    pub description: Option<&'a str>,
    pub coordinates: Option<&'a str>,
    pub location: Option<&'a str>,
}

/// Create user record in database
pub fn create_user(conn: &PgConnection, user: &NewUser) -> UserInfo {
    use db::schema::users;

    let user: User = diesel::insert_into(users::table)
        .values(user)
        .get_result(conn)
        .expect("Error creating user"); // Todo: error handling

    user.into()
}

impl<'a, 'r> FromRequest<'a, 'r> for UserInfo {
    type Error = ();

    /// Request guard for user authentication
    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let token = match request.headers().get_one("Authorization") {
            Some(jwt) => jwt,
            None => return Outcome::Failure((Status::Unauthorized, ())),
        };

        // Todo: secret key
        // Todo: error matching
        let token = match decode::<UserInfo>(token, b"secret", &Validation::default()) {
            Ok(token) => token,
            Err(_) => return Outcome::Failure((Status::Unauthorized, ())),
        };

        Outcome::Success(token.claims)
    }
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        let User {
            id,
            username,
            email,
            date,
            ..
        } = user;

        UserInfo {
            id,
            username,
            email,
            date,
        }
    }
}
