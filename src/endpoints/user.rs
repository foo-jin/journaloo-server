use bcrypt::{self, DEFAULT_COST};
use db::DbConn;
use db::models::user::{self, NewUser, User, UserInfo};
use diesel::prelude::*;
use jwt::{self, Header};
//use lettre::smtp::{SmtpTransport, SmtpTransportBuilder};
//use lettre::smtp::authentication::Mechanism;
//use lettre::smtp::SUBMISSION_PORT;
//use lettre_email::EmailBuilder;
use rocket::http::Status;
use rocket_contrib::Json;
use super::{log_db_err, log_err};
use rocket::response::status;

/// Registers a new user.
/// If the username or email is taken, fails with a `BadRequest` status.
/// If an unexpected error occurs, fails with an `InternalServiceError` status.
#[post("/user", format = "application/json", data = "<user>")]
pub fn signup(user: Json<NewUser>, conn: DbConn) -> Result<status::Created<String>, Status> {
    use db::schema::users::dsl::*;
    use diesel::result::Error;

    let mut user = user.into_inner();
    hash_password(&mut user).map_err(log_err)?;

    match users
        .filter(username.eq(&user.username))
        .or_filter(email.eq(&user.email))
        .first::<User>(&*conn)
    {
        Err(Error::NotFound) => (),
        Ok(v) => {
            debug!("duplicate user found: {:?}", v);
            return Err(Status::BadRequest);
        }
        Err(e) => return Err(log_err(e)),
    }

    let user_info = user::create(&user, &conn).map_err(log_err)?;
    let token = issue_token(&user_info).map_err(log_err)?;

    Ok(status::Created(String::new(), Some(token)))
}

/// Updates an existing user.
/// If unexpected errors occur, fails with an `InternalServiceError` status.
#[put("/user", format = "application/json", data = "<updated_user>")]
pub fn update(
    old_user: UserInfo,
    updated_user: Json<NewUser>,
    conn: DbConn,
) -> Result<String, Status> {
    let mut updated_user = updated_user.into_inner();
    hash_password(&mut updated_user).map_err(log_err)?;

    let user_info = user::update(&old_user, &updated_user, &*conn).map_err(log_err)?;
    let token = issue_token(&user_info).map_err(log_err)?;

    Ok(token)
}

/// Deletes a user, along with all its journeys and entries.
/// If an unexpected errors occur, fails with an `InternalServiceError` status.
#[delete("/user")]
pub fn delete(user: UserInfo, conn: DbConn) -> Result<(), Status> {
    user::delete(user, &*conn).map_err(log_err)
}

/// Login details of a user
#[derive(Serialize, Deserialize)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}

/// Grants an auth token to a user if the credentials match.
/// If the user does not exist, fails with a `NotFound` status.
/// If the credentials do not match, fails with an `Unauthorized` status.
/// If an unexpected errors occur, fails with an `InternalServiceError` status.
#[post("/user/login", format = "application/json", data = "<user_login>")]
pub fn login(user_login: Json<UserLogin>, conn: DbConn) -> Result<String, Status> {
    use db::schema::users::dsl::*;

    let user = users
        .filter(username.eq(&user_login.username))
        .first::<User>(&*conn)
        .map_err(log_db_err)?;

    if !bcrypt::verify(&user_login.password, &user.password).map_err(log_err)? {
        debug!("couldn't verify password");
        return Err(Status::Unauthorized);
    }

    let token = issue_token(&user.into()).map_err(log_err)?;

    Ok(token)
}

/// Reset a user's password. Details TBD
/// If an unexpected errors occur, fails with an `InternalServiceError` status.
#[put("/user/reset_password/<email_address>")]
#[allow(unused_variables)]
pub fn reset_password(
    email_address: String,
    conn: DbConn,
) -> Result<status::Accepted<String>, Status> {
    // Todo: implement reset_password
    unimplemented!()
}

/// Get a user by user ID.
/// If the user does not exist, fails with a `NotFound` status.
/// If an unexpected errors occur, fails with an `InternalServiceError` status.
#[get("/user/<user_id>")]
pub fn get_by_id(user_id: i32, conn: DbConn) -> Result<Json<UserInfo>, Status> {
    use db::schema::users::dsl::*;

    let user = users
        .find(user_id)
        .first::<User>(&*conn)
        .map_err(log_db_err)?;

    Ok(Json(user.into()))
}

/// Hash and salt a password.
fn hash_password(user: &mut NewUser) -> Result<(), bcrypt::BcryptError> {
    debug!("hashing password");
    user.password = bcrypt::hash(&user.password, DEFAULT_COST)?;
    Ok(())
}

/// Create an auth token containing a user's account details.
fn issue_token(user_info: &UserInfo) -> jwt::errors::Result<String> {
    use SECRET;
    debug!("creating token");
    jwt::encode(&Header::default(), user_info, SECRET.as_bytes())
}
