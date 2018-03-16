use chrono::NaiveDateTime;
use db::schema::entries;
use db::models::journey::Journey;

#[derive(Queryable, Identifiable, Associations)]
#[table_name = "entries"]
#[belongs_to(Journey)]
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
