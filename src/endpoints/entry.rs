#![allow(unused_variables)]

use db::DbConn;
use db::models::entry::{Entry, NewEntry};
use db::models::user::UserInfo;
use diesel::prelude::*;
use rocket::http::Status;
use rocket_contrib::Json;
use rocket::response::status;

/// Creates a new entry.
/// If the journey does not exist, fails with a `NotFound` status.
/// If the journey has ended already, fails with a `BadRequest` status.
/// If an unexpected error occurs, fails with an `InternalServiceError` status.
#[post("/entry", format = "application/json", data = "<entry>")]
pub fn create(
    entry: Json<NewEntry>,
    user: UserInfo,
    conn: DbConn,
) -> Result<status::Created<Json<Entry>>, Status> {
    // Todo: verify journey
    // Todo: create entry
    // Todo: return entry
    unimplemented!()
}

/// Deletes an entry.
/// If the entry does not exist, fails with a `NotFound` status.
/// If an unexpected error occurs, fails with an `InternalServiceError` status.
#[delete("/entry/<entry_id>")]
pub fn delete(entry_id: i32) -> Result<(), Status> {
    // Todo: verify entry_id
    // Todo: mark entry as archived
    unimplemented!()
}

#[derive(FromForm)]
pub struct Page {
    nr: u32,
}

/// Gets a page of global entries.
/// If an unexpected error occurs, fails with an `InternalServiceError` status.
#[get("/entry?<page>")]
pub fn get_all(page: Page) -> Result<Json<Vec<Entry>>, Status> {
    let page = page.nr;
    // Todo: fetch page
    // Todo: return page
    unimplemented!()
}

/// Gets a page of a journey's entries.
/// If the journey does not exist, fails with a `NotFound` status.
/// If an unexpected error occurs, fails with an `InteralServiceError` status.
#[get("/entry/<journey_id>?<page>")]
pub fn get_by_journey(journey_id: i32, page: Page) -> Result<Json<Vec<Entry>>, Status> {
    let page = page.nr;
    // Todo: verify journey
    // Todo: fetch page
    // Todo: return page
    unimplemented!()
}
