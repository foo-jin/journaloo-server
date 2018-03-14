use bcrypt::{DEFAULT_COST, hash};
use db::DbConn;
use db::models::user;
use jwt::{encode, Header};
use rocket::http::Status;
use rocket_contrib::Json;

// Todo: request guard checking for validity of user data
#[post("/user", format = "application/json", data = "<user>")]
pub fn signup(user: Json<user::NewUser>, conn: DbConn) -> Result<String, Status> {
    let mut user = user.into_inner();
    user.password = hash(&user.password, DEFAULT_COST).map_err(|_| Status::InternalServerError)?;
    let user_info = user::create(&conn, &user);
    let token = encode(&Header::default(), &user_info, "secret".as_ref()) // Todo: secret key
        .map_err(|_| Status::InternalServerError)?;

    Ok(token)
}
