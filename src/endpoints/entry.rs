use diesel::prelude::*;
use rocket::http::Status;
use rocket::response::status;
use rocket_contrib::Json;

use super::{log_db_err, ErrStatus};
use db::DbConn;
use db::models::entry::{self, Entry, NewEntry};
use db::models::journey::Journey;
use db::models::user::UserInfo;
use endpoints::PAGE_SIZE;
use endpoints::PageQuery;
use endpoints::Page;

/// Creates a new entry.
/// If the journey does not exist, fails with a `NotFound` status.
/// If the journey has ended already, fails with a `BadRequest` status.
/// If an unexpected error occurs, fails with an `InternalServiceError` status.
#[post("/entry", format = "application/json", data = "<new_entry>")]
pub fn create(
    new_entry: Json<NewEntry>,
    _user: UserInfo,
    conn: DbConn,
) -> Result<status::Created<Json<Entry>>, ErrStatus> {
    use db::schema::journeys;

    let journey = journeys::table
        .find(new_entry.journey_id)
        .first::<Journey>(&*conn)
        .map_err(log_db_err)?;

    if journey.end_date.is_some() {
        return Err(status::Custom(Status::BadRequest, ()));
    }

    let entry = entry::create(&new_entry, &*conn).map_err(log_db_err)?;

    Ok(status::Created(String::new(), Some(Json(entry))))
}

/// Deletes an entry.
/// If the entry does not exist, fails with a `NotFound` status.
/// If an unexpected error occurs, fails with an `InternalServiceError` status.
#[delete("/entry/<entry_id>")]
pub fn delete(entry_id: i32, conn: DbConn) -> Result<(), ErrStatus> {
    entry::archive(entry_id, &*conn).map_err(log_db_err)
}

#[derive(FromForm)]
pub struct EntryQuery {
    page: Page,
    journey: Option<i32>,
}

// Note: `offset` usage here has bad performance on large page numbers
/// Gets a page of entries according to the query-string.
/// If a nonexistent journey ID is given, fails with a `NotFound` status.
/// If an unexpected error occurs, fails with an `InternalServiceError` status.
#[get("/entry/all?<query>")]
pub fn get_all(query: EntryQuery, conn: DbConn) -> Result<Json<Vec<Entry>>, ErrStatus> {
    use db::schema::entries;
    let page = query.page.0;

    let mut target = entries::table
        .order(entries::created.desc())
        .filter(entries::archived.eq(false)).into_boxed();

    if let Some(jid) = query.journey {
        target = target.filter(entries::journey_id.eq(jid));
    }

    let result = target
        .offset(page * PAGE_SIZE)
        .limit(PAGE_SIZE)
        .get_results::<Entry>(&*conn)
        .map_err(log_db_err)?;

    Ok(Json(result))
}
