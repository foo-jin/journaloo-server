use bcrypt::{self, hash, DEFAULT_COST};
use db::DbConn;
use db::models::user::{self, NewUser, User, UserInfo};
use jwt::{encode, Header};
use rocket::http::Status;
use rocket_contrib::Json;
use std::fmt::Debug;

fn log_error<T: Debug>(e: T) -> Status {
    error!("Encountered error: {:?}", e);
    Status::InternalServerError
}

fn hash_password(user: &mut NewUser) -> Result<(), bcrypt::BcryptError> {
    user.password = hash(&user.password, DEFAULT_COST)?;
    Ok(())
}

/// Register a new user.
/// Will return `Status::BadRequest` on conflicting username or email.
#[post("/user", format = "application/json", data = "<user>")]
pub fn signup(user: Json<NewUser>, conn: DbConn) -> Result<String, Status> {
    use diesel::prelude::*;
    use diesel::result::Error;
    use db::schema::users::dsl::*;

    let mut user = user.into_inner();
    hash_password(&mut user).map_err(log_error)?;

    // check for duplicate users
    match users
        .filter(username.eq(&user.username))
        .or_filter(email.eq(&user.email))
        .first::<User>(&*conn)
        {
            Ok(v) => {
                debug!("user found: {:?}", v);
                return Err(Status::BadRequest);
            }
            Err(Error::NotFound) => (),
            Err(e) => return Err(log_error(e)),
        }

    let user_info = user::create(&conn, &user).map_err(log_error)?;
    // Todo: secret key
    let token = encode(&Header::default(), &user_info, "secret".as_ref()).map_err(log_error)?;

    Ok(token)
}

#[put("/user", format = "application/json", data = "<updated_user>")]
pub fn update(old_user: UserInfo, updated_user: Json<NewUser>, conn: DbConn) -> Result<(), Status> {
    use diesel;
    use diesel::prelude::*;
    use db::schema::users::dsl::*;

    let mut updated_user = updated_user.into_inner();
    hash_password(&mut updated_user).map_err(log_error)?;

    let target = users.find(old_user.id);
    diesel::update(target)
        .set(&updated_user)
        .execute(&*conn)
        .map_err(log_error)?;

    Ok(())
}
