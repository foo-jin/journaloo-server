use diesel;
use diesel::dsl::now;
use diesel::prelude::*;
use rocket_contrib::Json;

use super::{log_db_err, ErrStatus, Page, PAGE_SIZE};
use db::DbConn;
use db::models::journey::{self, Journey, NewJourney};
use db::models::user::UserInfo;

use chrono::DateTime;
use chrono::FixedOffset;
use rocket::response::status;

#[post("/journey", format = "application/json", data = "<journey>")]
pub fn create(
    journey: Json<NewJourney>,
    conn: DbConn,
) -> Result<status::Created<Json<TimezoneJourney>>, ErrStatus> {
    let journey = journey.into_inner();
    let journey = journey::create(&conn, &journey).map_err(log_db_err)?;

    Ok(status::Created(
        String::new(),
        Some(Json(journey.into())),
    ))
}

/// Return a Json Journey object of the journey that matches the id
#[get("/journey/<jid>")]
pub fn get_by_id(
    jid: i32,
    conn: DbConn,
) -> Result<Json<TimezoneJourney>, ErrStatus> {
    use db::schema::journeys::dsl::*;

    let journey: Journey = journeys
        .find(&jid)
        .first(&*conn)
        .map_err(log_db_err)?;

    Ok(Json(journey.into()))
}

/// Set a journey status to "archived", simulating deletion
#[delete("/journey/<jid>")]
pub fn delete(jid: i32, user: UserInfo, conn: DbConn) -> Result<(), ErrStatus> {
    use db::schema::journeys::dsl::*;

    diesel::update(journeys.filter(user_id.eq(user.id)).find(jid))
        .set(archived.eq(true))
        .execute(&*conn)
        .map_err(log_db_err)?;

    Ok(())
}

/// Update the journey that matches the passed id
#[put("/journey", format = "application/json", data = "<journey>")]
pub fn update(journey: Json<Journey>, conn: DbConn) -> Result<(), ErrStatus> {
    use db::schema::journeys::dsl::*;

    let journey = journey.into_inner();
    diesel::update(journeys.find(journey.id))
        .set(title.eq(journey.title))
        .execute(&*conn)
        .map_err(log_db_err)?;

    Ok(())
}

#[derive(FromForm)]
pub struct JourneyQuery {
    page: Page,
}

/// Get the journeys of a user
#[get("/journey/user/<uid>?<page>")]
pub fn get_journeys_by_user(
    uid: i32,
    page: JourneyQuery,
    conn: DbConn,
) -> Result<Json<Vec<TimezoneJourney>>, ErrStatus> {
    use db::schema::journeys::dsl::*;
    let page = page.page.0;

    let result = journeys
        .order(start_date.desc())
        .filter(user_id.eq(uid))
        .filter(archived.eq(false))
        .offset(page * PAGE_SIZE)
        .limit(PAGE_SIZE)
        .get_results::<Journey>(&*conn)
        .map_err(log_db_err)?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(Json(result))
}

/// Get the current active journey of a user
#[get("/journey/<uid>/active")]
pub fn get_active_journey_by_user(
    uid: i32,
    conn: DbConn,
) -> Result<Json<TimezoneJourney>, ErrStatus> {
    use db::schema::journeys::dsl::*;

    let result = journeys
        .order(start_date.desc())
        .filter(user_id.eq(uid))
        .filter(archived.eq(false))
        .filter(end_date.is_null())
        .first::<Journey>(&*conn)
        .map_err(log_db_err)?;

    Ok(Json(result.into()))
}

/// Updates the end_date field of a journey.
/// If the journey does not exist, or has ended already, fails with a
/// `NotFound` status. If an unexpected error occurs, fails with an
/// `InternalServerError` status.
#[put("/journey/<jid>/end")]
pub fn end(
    jid: i32,
    _user: UserInfo,
    conn: DbConn,
) -> Result<Json<TimezoneJourney>, ErrStatus> {
    use db::schema::journeys;

    let target = journeys::table
        .find(jid)
        .filter(journeys::archived.eq(false));

    let result = diesel::update(target)
        .set(journeys::end_date.eq(now.nullable()))
        .get_result::<Journey>(&*conn)
        .map_err(log_db_err)?;

    Ok(Json(result.into()))
}

#[derive(Serialize)]
pub struct TimezoneJourney {
    id: i32,
    user_id: i32,
    title: String,
    archived: bool,
    start_date: DateTime<FixedOffset>,
    end_date: Option<DateTime<FixedOffset>>,
}

impl From<Journey> for TimezoneJourney {
    fn from(journey: Journey) -> Self {
        let Journey {
            id,
            user_id,
            title,
            archived,
            start_date,
            end_date,
        } = journey;

        let hour = 3600;
        let start_date =
            DateTime::from_utc(start_date, FixedOffset::east(2 * hour));
        let end_date = end_date
            .map(|date| DateTime::from_utc(date, FixedOffset::east(2 * hour)));

        TimezoneJourney {
            id,
            user_id,
            title,
            archived,
            start_date,
            end_date,
        }
    }
}
