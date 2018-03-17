#![allow(unused_variables)]

use db::DbConn;
use db::models::entry::{Entry, NewEntry};
use db::models::user::UserInfo;
use diesel::prelude::*;
use rocket::http::Status;
use rocket_contrib::Json;
use rocket::response::status;
use super::{log_db_err, log_err};
use db::models::journey::Journey;
use db::models::entry;

/// Creates a new entry.
/// If the journey does not exist, fails with a `NotFound` status.
/// If the journey has ended already, fails with a `BadRequest` status.
/// If an unexpected error occurs, fails with an `InternalServiceError` status.
#[post("/entry", format = "application/json", data = "<new_entry>")]
pub fn create(
    new_entry: Json<NewEntry>,
    user: UserInfo,
    conn: DbConn,
) -> Result<status::Created<Json<Entry>>, Status> {
    use db::schema::journeys::dsl::*;
    debug!("create endpoint called");

    debug!("verifying journey");
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
    nr: u32,
}

/// Gets a page of global entries.
/// If an unexpected error occurs, fails with an `InternalServiceError` status.
#[get("/entry?<page>")]
pub fn get_all(page: Page, conn: DbConn) -> Result<Json<Vec<Entry>>, Status> {
    let page = page.nr;
    // Todo: fetch page
    // Todo: return page
    unimplemented!()
}

/// Gets a page of a journey's entries.
/// If the journey does not exist, fails with a `NotFound` status.
/// If an unexpected error occurs, fails with an `InteralServiceError` status.
#[get("/entry/<journey_id>?<page>")]
pub fn get_by_journey(journey_id: i32, page: Page, conn: DbConn) -> Result<Json<Vec<Entry>>, Status> {
    let page = page.nr;
    // Todo: verify journey
    // Todo: fetch page
    // Todo: return page
    unimplemented!()
}
