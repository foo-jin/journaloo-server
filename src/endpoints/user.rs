use bcrypt::{DEFAULT_COST, hash, verify};
use db::DbConn;
use db::models::{create_user, NewUser};
use jwt::{decode, encode, Header};
use rocket::http::Status;
use rocket_contrib::Json;

#[post("/user", format = "application/json", data = "<user>")]
pub fn signup(user: Json<NewUser>, conn: DbConn) -> Result<String, Status> {
    let mut user = user.into_inner();
    user.password = hash(&user.password, DEFAULT_COST)
        .map_err(|_| Status::InternalServerError)?;
    let user_info = create_user(&conn, user); // Todo: error handling
    let token = encode(&Header::default(), &user_info, "secret".as_ref()) // Todo: secret key
        .map_err(|_| Status::InternalServerError)?;

    Ok(token)
}
