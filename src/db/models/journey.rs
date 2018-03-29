use chrono::NaiveDateTime;
use db::models::user::UserInfo;
use db::schema::journeys;
use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;

#[derive(Queryable, Identifiable, Associations, Debug, Serialize,
         Deserialize)]
#[belongs_to(UserInfo, foreign_key = "user_id")]
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
pub fn create(
    conn: &PgConnection,
    journey: &NewJourney,
) -> diesel::QueryResult<Journey> {
    diesel::insert_into(journeys::table)
        .values(journey)
        .get_result::<Journey>(conn)
}
// TODO: transfer complexity to models from endpoints
