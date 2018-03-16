use chrono::NaiveDateTime;
use db::schema::entries;
use db::models::journey::Journey;

#[derive(Queryable, Identifiable, Associations, Serialize)]
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

#[derive(Insertable, Deserialize)]
#[table_name = "entries"]
pub struct NewEntry {
    pub journey_id: i32,
    pub description: Option<String>,
    pub coordinates: Option<String>,
    pub location: Option<String>,
}
