use db::DbConn;
use db::models::{NewUser, User};

use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use rocket::response::{content, status};
use rocket::http::Status;
use rocket_contrib::Json;

/// Create user record in database
fn create_user<'a>(conn: &PgConnection, new_user: NewUser<'a>) -> User {
    use db::schema::users;

    diesel::insert_into(users::table)
        .values(&new_user)
        .get_result(conn)
        .expect("Error creating user") // Todo: error handling
}

#[post("/user")]
pub fn signup(conn: DbConn) -> Result<Json, Status> {
    unimplemented!()
}
