use db::DbConn;
use db::models::journey::{self, NewJourney, Journey};
use db::schema::users::dsl::*; ///todo ???
use diesel;
use diesel::prelude::*;
use jwt::{self, Header};
use lettre::smtp::{SmtpTransport, SmtpTransportBuilder};
use lettre::smtp::authentication::Mechanism;
use lettre::smtp::SUBMISSION_PORT;
use lettre_email::EmailBuilder;
use rocket::http::Status;
use rocket_contrib::Json;
use std::fmt::Debug;
use rocket::response::status;

#[post("/journey", format = "application/json", data = "<journey>")]
pub fn create_journey(journey: Json<NewJourney>, conn: DbConn) -> Result<(), Status> {
    let mut journey = journey.into_inner();
    journey::create(&conn, &journey);

    Ok(())
}

/// Return a Json Journey object of the journey that matches the id
#[get("/journey/<id>", format = "application/json")]
pub fn return_journey(id: i64, conn: DbConn) -> Result<Json<Journey>, diesel::result::Error> {
    let mut journey = journeys::table.find(&id).first(&conn);
    Ok(Json(journey))
}

