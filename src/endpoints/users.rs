use db::DbConn;
use db::models::{NewUser, User};

use diesel;
use diesel::pg::PgConnection;
use rocket::response::{content, status};
use rocket::http::Status;
use rocket_contrib::Json;

fn create_user<'a>(conn: &PgConnection, new_user: NewUser<'a>) -> User {
    use schema::users;

    diesel::insert_into(users::table)
        .values(&new_user)
        .get_result(conn)
        .expect("Error creating user") // Todo: error handling
}

#[post("/user")]
fn signup(conn: DbConn) -> &'static str {
    "asdf"
}
