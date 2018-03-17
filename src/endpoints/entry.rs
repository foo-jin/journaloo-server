use db::DbConn;
use db::models::entry::{self, Entry, NewEntry};
use db::models::journey::Journey;
use db::models::user::UserInfo;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::response::status;
use rocket_contrib::Json;
use super::log_db_err;

const PAGE_SIZE: i64 = 10;

/// Creates a new entry.
/// If the journey does not exist, fails with a `NotFound` status.
/// If the journey has ended already, fails with a `BadRequest` status.
/// If an unexpected error occurs, fails with an `InternalServiceError` status.
#[post("/entry", format = "application/json", data = "<new_entry>")]
pub fn create(
    new_entry: Json<NewEntry>,
    _user: UserInfo,
    conn: DbConn,
) -> Result<status::Created<Json<Entry>>, Status> {
    use db::schema::journeys::dsl::*;

    let journey = journeys
        .find(new_entry.journey_id)
        .first::<Journey>(&*conn)
        .map_err(log_db_err)?;

    if journey.end_date.is_some() {
        return Err(Status::BadRequest);
    }

    let entry = entry::create(&new_entry, &*conn).map_err(log_db_err)?;

    Ok(status::Created(String::new(), Some(Json(entry))))
}

/// Deletes an entry.
/// If the entry does not exist, fails with a `NotFound` status.
/// If an unexpected error occurs, fails with an `InternalServiceError` status.
#[delete("/entry/<entry_id>")]
pub fn delete(entry_id: i32, conn: DbConn) -> Result<(), Status> {
    entry::archive(entry_id, &*conn).map_err(log_db_err)
}

#[derive(FromForm)]
pub struct Page {
    page: i64,
}

// Todo: verify that offset and limit do not cause errors if they overshoot the total.
// Note: `offset` usage here has bad performance on large page numbers
/// Gets a page of global entries.
/// If an unexpected error occurs, fails with an `InternalServiceError` status.
#[get("/entry?<page>")]
pub fn get_all(page: Page, conn: DbConn) -> Result<Json<Vec<Entry>>, Status> {
    use db::schema::entries::dsl::*;
    let page = page.page;

    let result = entries
        .order(created.desc())
        .offset(page * PAGE_SIZE)
        .limit(page * (PAGE_SIZE + 1))
        .get_results::<Entry>(&*conn)
        .map_err(log_db_err)?;

    Ok(Json(result))
}

/// Gets a page of a specific journey's entries.
/// If the journey does not exist, fails with a `NotFound` status.
/// If an unexpected error occurs, fails with an `InteralServiceError` status.
#[get("/entry/<jid>?<page>")]
pub fn get_by_journey(jid: i32, page: Page, conn: DbConn) -> Result<Json<Vec<Entry>>, Status> {
    use db::schema::entries::dsl::*;
    let page = page.page;

    let result = entries
        .order(created.desc())
        .filter(journey_id.eq(jid))
        .filter(archived.eq(false))
        .offset(page * PAGE_SIZE)
        .limit(PAGE_SIZE)
        .get_results::<Entry>(&*conn)
        .map_err(log_db_err)?;

    Ok(Json(result))
}
