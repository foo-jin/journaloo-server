use chrono::NaiveDateTime;
use db::schema::journeys;
use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use jwt::{decode, Validation};
use rocket::http::Status;
use rocket::Outcome;
use rocket::request::{self, FromRequest};
use rocket::Request;

#[derive(Queryable)]
pub struct Journey {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub archived: bool,
    pub start_date: NaiveDateTime,
    pub end_date: Option<NaiveDateTime>,
}

#[derive(Insertable, Deserialize)]
#[table_name = "journeys"]
pub struct NewJourney {
    pub user_id: i32,
    pub title: String,
}

/// inserts journey into database
pub fn create(conn: &PgConnection, journey: &NewJourney ) { //TODO: add return type for testing

    diesel::insert_into(journeys::table)
        .values(journey);
        //.get_result::<Journey>(conn);
}