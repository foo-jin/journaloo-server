use chrono::NaiveDateTime;
use db::schema::journeys;
use db::models::user::UserInfo;

#[derive(Queryable, Identifiable, Associations)]
#[belongs_to(UserInfo, foreign_key = "user_id")]
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
