use super::schema::users;
use chrono::NaiveDateTime;

#[derive(Queryable)]
#[primary_key(id)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub date: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub email: &'a str,
    pub password: String,
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