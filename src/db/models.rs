use chrono::NaiveDateTime;
use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use super::schema::*;

#[derive(Queryable)]
#[primary_key(id)]
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
#[primary_key(id)]
#[table_name = "users"]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub date: NaiveDateTime,
}

#[derive(Queryable)]
#[primary_key(id)]
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
#[primary_key(id)]
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
pub fn create_user<'a>(conn: &PgConnection, user: NewUser) -> UserInfo {
    use db::schema::users;

    let user: User = diesel::insert_into(users::table)
        .values(&user)
        .get_result(conn)
        .expect("Error creating user"); // Todo: error handling

    user.into()
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        let User { id, username, email, date, .. } = user;
        UserInfo {
            id,
            username,
            email,
            date,
        }
    }
}