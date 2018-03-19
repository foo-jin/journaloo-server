use diesel::result::Error;
use rocket::http::Status;
use std::fmt::Debug;

pub mod user;
pub mod journey;
pub mod entry;

/// Logs an error with error priority.
/// Returns a `InternalServiceError` status.
fn log_err<T: Debug>(e: T) -> Status {
    error!("Encountered error -- {:?}", e);
    Status::InternalServerError
}

/// Logs a diesel error with error priority.
/// If the error was `NotFound`, returns a `NotFound` status.
/// Else, returns an `InternalServiceError` status.
fn log_db_err(e: Error) -> Status {
    match e {
        Error::NotFound => Status::NotFound,
        e => log_err(e),
    }
}
