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
#[table_name="users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub email: &'a str,
    pub password: String,
}

#[derive(Queryable)]
#[primary_key(id)]
pub struct Journey {
    pub id: i32,
    pub userid: i32,
    pub title: String,
    pub archived: bool,
    pub start_date: NaiveDateTime,
    pub end_date: Option<NaiveDateTime>
}

#[derive(Insertable)]
#[table_name="users"]
pub struct NewJourney<'a> {
    pub userid: i32,
    pub title: &'a str,
}