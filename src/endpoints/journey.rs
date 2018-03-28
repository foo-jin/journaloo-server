use diesel::prelude::*;
use rocket_contrib::Json;
use diesel::dsl::now;
use diesel;

use super::{log_db_err, ErrStatus};
use db::DbConn;
use db::models::journey::Journey;
use db::models::user::UserInfo;
use db::DbConn;
use db::models::journey::{self, Journey, NewJourney};
use db::schema::journeys;
use diesel;
use diesel::prelude::*;
use jwt::{self, Header};
use rocket::http::Status;
use rocket_contrib::Json;
use rocket::response::status;
use super::{log_db_err, Page, PAGE_SIZE};
use db::models::user::UserInfo;

#[post("/journey", format = "application/json", data = "<journey>")]
pub fn create(
    journey: Json<NewJourney>,
    conn: DbConn,
) -> Result<status::Created<Json<Journey>>, Status> {
    let mut journey = journey.into_inner();
    let journey = journey::create(&conn, &journey).map_err(log_db_err)?;

    Ok(status::Created(String::new(), Some(Json(journey)))) // TODO: return created, like create entry
}

/// Return a Json Journey object of the journey that matches the id
#[get("/journey/<jid>")]
pub fn get_by_id(jid: i32, conn: DbConn) -> Result<Json<Journey>, Status> {
    use db::schema::journeys::dsl::*;

    let journey = journeys.find(&jid).first(&*conn).map_err(log_db_err)?;
    Ok(Json(journey))
}

/// Set a journey status to "archived", simulating deletion
#[delete("/journey/<jid>")]
pub fn delete(jid: i32, user: UserInfo, conn: DbConn) -> Result<(), Status> {
    use db::schema::journeys::dsl::*;

    diesel::update(journeys.filter(user_id.eq(user.id)).find(jid))
        .set(archived.eq(true))
        .execute(&*conn)
        .map_err(log_db_err)?;
    Ok(())
}

/// Update the journey that matches the passed id
#[put("/journey", format = "application/json", data = "<journey>")]
pub fn update(journey: Json<Journey>, conn: DbConn) -> Result<(), Status> {
    use db::schema::journeys::dsl::*;

    let journey = journey.into_inner();
    diesel::update(journeys.find(journey.id))
        .set(title.eq(journey.title))
        .execute(&*conn)
        .map_err(log_db_err)?;
    Ok(())
}

/// Get the journeys of a user
#[get("/journey/user/<uid>?<page>")]
pub fn get_journeys_by_user(
    uid: i32,
    page: Page,
    conn: DbConn,
) -> Result<Json<Vec<Journey>>, Status> {
    use db::schema::journeys::dsl::*;
    let page = page.page;

    let result = journeys
        .order(start_date.desc())
        .filter(user_id.eq(uid))
        .filter(archived.eq(false))
        .offset(page * PAGE_SIZE)
        .limit(page * (PAGE_SIZE + 1))
        .get_results::<Journey>(&*conn)
        .map_err(log_db_err)?;

    Ok(Json(result))
}

/// Get the current active journey of a user
#[get("/journey/<uid>/active")]
pub fn get_active_journey_by_user(uid: i32, conn: DbConn) -> Result<Json<Journey>, Status> {
    use db::schema::journeys::dsl::*;

    let result = journeys
        .order(start_date.desc())
        .filter(user_id.eq(uid))
        .filter(archived.eq(false))
        .filter(end_date.is_null())
        .first::<Journey>(&*conn)
        .map_err(log_db_err)?;

    Ok(Json(result))
}

/// Updates the end_date field of a journey.
/// If the journey does not exist, or has ended already, fails with a `NotFound` status.
/// If an unexpected error occurs, fails with an `InternalServerError` status.
#[put("/journey/<jid>/end")]
pub fn end(jid: i32, _user: UserInfo, conn: DbConn) -> Result<Json<Journey>, ErrStatus> {
    use db::schema::journeys;

    let target = journeys::table.find(jid).filter(journeys::archived.eq(false));
    let result = diesel::update(target)
        .set(journeys::end_date.eq(now.nullable()))
        .get_result::<Journey>(&*conn)
        .map_err(log_db_err)?;

    Ok(Json(result))
}