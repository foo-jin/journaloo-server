use db::Conn;
use rocket::response::{content, status};
use rocket::http::Status;
use rocket_contrib::Json;

struct User;

#[post("/user")]
fn signup(user: User, conn: Conn) -> Result<Json<String>, Status> {
    "Hello, world!"
}