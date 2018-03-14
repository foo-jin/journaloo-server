use bcrypt::{hash, DEFAULT_COST};
use db::DbConn;
use db::models::user;
use jwt::{encode, Header};
use rocket::http::Status;
use rocket_contrib::Json;

#[post("/user", format = "application/json", data = "<user>")]
pub fn signup(user: Json<user::NewUser>, conn: DbConn) -> Result<String, Status> {
    use diesel::prelude::*;
    use diesel::result::Error;
    use db::schema::users::dsl::*;

    let mut user = user.into_inner();
    user.password = hash(&user.password, DEFAULT_COST).map_err(|_| Status::InternalServerError)?;

    // check for duplicate users
    match users
        .filter(username.eq(&user.username))
        .or_filter(email.eq(&user.email))
        .first::<user::User>(&*conn)
        {
            Ok(_) => return Err(Status::BadRequest),
            Err(Error::NotFound) => (),
            _ => return Err(Status::InternalServerError),
        }

    let user_info = user::create(&conn, &user).map_err(|_| Status::InternalServerError)?;

    // Todo: secret key
    let token = encode(&Header::default(), &user_info, "secret".as_ref())
        .map_err(|_| Status::InternalServerError)?;

    Ok(token)
}
