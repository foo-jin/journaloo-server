use diesel::result::Error;
use rocket::http::Status;
use rocket::response::status;
use std::fmt::Debug;

pub mod user;
pub mod journey;
pub mod entry;

type ErrStatus = status::Custom<()>;

/// Logs an error with error priority.
/// Returns a `InternalServiceError` status.
fn log_err<T: Debug>(e: T) -> ErrStatus {
    error!("Encountered error -- {:?}", e);
    status::Custom(Status::InternalServerError, ())
}

/// Logs a diesel error with error priority.
/// If the error was `NotFound`, returns a `NotFound` status.
/// Else, returns an `InternalServiceError` status.
fn log_db_err(e: Error) -> ErrStatus {
    match e {
        Error::NotFound => status::Custom(Status::NotFound, ()),
        e => log_err(e),
    }
}
