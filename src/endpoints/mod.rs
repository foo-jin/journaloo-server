use std::fmt::Debug;

use diesel::result::Error;
use rocket::http::Status;
use rocket::response::status;
use rocket::request::FromFormValue;
use rocket::http::RawStr;

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

const PAGE_SIZE: i64 = 10;

struct Page(i64);

impl<'v> FromFormValue<'v> for Page {
    type Error = &'v RawStr;

    /// Parses page number from forms.
    /// Will fail if the the page number is smaller than zero.
    fn from_form_value(form_value: &'v RawStr) -> Result<Self, &'v RawStr> {
        match form_value.parse::<i64>() {
            Ok(page) if page >= 0 => Ok(Page(page)),
            _ => Err(form_value)
        }
    }
}


