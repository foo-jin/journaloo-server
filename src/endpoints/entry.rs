use diesel::prelude::*;
use rocket::http::Status;
use rocket::response::NamedFile;
use rocket::response::status;
use rocket_contrib::Json;
use std::fs;
use std::path::Path;

use diesel;

use super::{log_err, log_db_err, ErrStatus, Page, PAGE_SIZE};
use db::DbConn;
use db::models::entry::{self, Entry, NewEntry};
use db::models::journey::Journey;
use db::models::user::UserInfo;
use db::schema::entries;
use rocket::Data;

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

/// Puts the image of an entry in the file system.
/// If an unexpected error occurs, fails with an `InternalServiceError` status.
#[post("/entry/<entry_id>/image", data = "<image>")]
pub fn create_image(
    entry_id: i32,
    image: Data,
    _user: UserInfo,
    conn: DbConn,
) -> Result<status::Created<()>, ErrStatus> {
    let entry = entries::table
        .find(entry_id)
        .first::<Entry>(&*conn)
        .map_err(log_db_err)?;

    let path_string = format!("{}/{}", entry.journey_id, entry.id);
    let path = Path::new(&path_string);
    let _file_folder = fs::create_dir(path); // TODO: match errors

    image.stream_to_file(path).map_err(log_err)?;

    Ok(status::Created(String::new(), Some(())))
}

/// Gets the data body of an entry.
/// If an unexpected error occurs, fails with an `InternalServiceError` status.
#[get("/entry/<entry_id>")]
pub fn get_by_id(entry_id: i32, conn: DbConn) -> Result<Json<Entry>, ErrStatus> {
    use db::schema::entries::dsl::*;

    let entry = entries.find(entry_id).first(&*conn).map_err(log_db_err)?;
    Ok(Json(entry))
}

/// Updates an entry.
/// Takes a NewEntry object.
/// If an unexpected error occurs, fails with an `InternalServiceError` status.
#[put("/entry/<entry_id>", format = "application/json", data = "<new_entry>")]
pub fn update(
    entry_id: i32,
    new_entry: Json<NewEntry>,
    _user: UserInfo,
    conn: DbConn,
) -> Result<(), ErrStatus> {
    use db::schema::entries::dsl::*;

    let entry = new_entry.into_inner();
    let target = entries.find(entry_id);

    diesel::update(target).set(description.eq(entry.description)).execute(&*conn).map_err(log_db_err)?;
    Ok(())
}

/// Retrieves the image of an entry.
/// If an unexpected error occurs, fails with an `InternalServiceError` status.
#[get("/entry/<entry_id>/image")]
pub fn get_image_by_id(entry_id: i32, conn: DbConn) -> Option<NamedFile> {
    let entry = entries::table
        .find(entry_id)
        .first::<Entry>(&*conn)
        .map_err(log_db_err)
        .ok()?;

    NamedFile::open(format!("{}/{}", entry.journey_id, entry.id)).ok()
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
        .filter(entries::archived.eq(false))
        .into_boxed();

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
