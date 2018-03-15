use bcrypt::{self, hash, DEFAULT_COST};
use db::DbConn;
use db::models::user::{self, NewUser, User, UserInfo};
use db::schema::users::dsl::*;
use diesel;
use diesel::prelude::*;
use jwt::{encode, Header};
use rocket::http::Status;
use rocket_contrib::Json;
use std::fmt::Debug;

fn log_err<T: Debug>(e: T) -> Status {
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
    use diesel::result::Error;

    let mut user = user.into_inner();
    hash_password(&mut user).map_err(log_err)?;

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
            Err(e) => return Err(log_err(e)),
        }

    let user_info = user::create(&conn, &user).map_err(log_err)?;
    // Todo: secret key
    let token = encode(&Header::default(), &user_info, "secret".as_ref()).map_err(log_err)?;

    Ok(token)
}

#[put("/user", format = "application/json", data = "<updated_user>")]
pub fn update(old_user: UserInfo, updated_user: Json<NewUser>, conn: DbConn) -> Result<(), Status> {
    let mut updated_user = updated_user.into_inner();
    hash_password(&mut updated_user).map_err(log_err)?;

    let target = users.find(old_user.id);
    let updated = diesel::update(target)
        .set(&updated_user)
        .execute(&*conn)
        .map_err(log_err)?;

    info!("updated {} users", updated);

    Ok(())
}

#[delete("/user")]
pub fn delete(user: UserInfo, conn: DbConn) -> Result<(), Status> {
    let target = users.find(user.id);
    let deleted = diesel::delete(target).execute(&*conn).map_err(log_err)?;

    info!("deleted {} users", deleted);

    Ok(())
}
