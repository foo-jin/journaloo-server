use bcrypt::{self, DEFAULT_COST};
use db::DbConn;
use db::models::user::{self, NewUser, User, UserInfo};
use diesel::prelude::*;
use jwt::{self, Header};
use lettre::smtp::{SmtpTransport, SmtpTransportBuilder};
use lettre::smtp::authentication::Mechanism;
use lettre::smtp::SUBMISSION_PORT;
use lettre_email::EmailBuilder;
use rocket::http::Status;
use rocket_contrib::Json;
use std::fmt::Debug;
use rocket::response::status;

/// Register a new user.
/// Will return `Status::BadRequest` on conflicting username or email.
#[post("/user", format = "application/json", data = "<user>")]
pub fn signup(user: Json<NewUser>, conn: DbConn) -> Result<status::Created<String>, Status> {
    use db::schema::users::dsl::*;
    use diesel::result::Error;
    debug!("signup endpoint called");

    let mut user = user.into_inner();
    hash_password(&mut user).map_err(log_err)?;

    debug!("checking for duplicate users");
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

/// Update an existing user
#[put("/user", format = "application/json", data = "<updated_user>")]
pub fn update(old_user: UserInfo, updated_user: Json<NewUser>, conn: DbConn) -> Result<(), Status> {
    debug!("update endpoint called");

    let mut updated_user = updated_user.into_inner();
    hash_password(&mut updated_user).map_err(log_err)?;

    user::update(old_user.id, updated_user, &*conn).map_err(log_err)
}

/// Delete a user, along with all its journeys and entries.
#[delete("/user")]
pub fn delete(user: UserInfo, conn: DbConn) -> Result<(), Status> {
    debug!("delete endpoint called");

    user::delete(user, &*conn).map_err(log_err)
}

/// Login details of a user
#[derive(Serialize, Deserialize)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}

/// Grant an auth token to a user if the credentials match.
#[post("/user/login", format = "application/json", data = "<user_login>")]
pub fn login(user_login: Json<UserLogin>, conn: DbConn) -> Result<String, Status> {
    use db::schema::users::dsl::*;
    use diesel::result::Error;
    debug!("login endpoint called");

    debug!("verifying user");
    let user = users
        .filter(username.eq(&user_login.username))
        .first::<User>(&*conn)
        .map_err(|e| match e {
            Error::NotFound => {
                debug!("user not found!");
                Status::NotFound
            }
            _ => log_err(e),
        })?;

    debug!("verifying password");
    if !bcrypt::verify(&user_login.password, &user.password).map_err(log_err)? {
        debug!("couldn't verify password");
        return Err(Status::Unauthorized);
    }

    let token = issue_token(&user.into()).map_err(log_err)?;

    Ok(token)
}

/// Reset a user's password. Details TBD
#[put("/user/reset_password/<email_address>")]
pub fn reset_password(
    email_address: String,
    conn: DbConn,
) -> Result<status::Accepted<String>, Status> {
    // Todo: implement reset_password
    unimplemented!()
}

/// Get a user by user ID
#[get("/user/<user_id>")]
pub fn get_by_id(user_id: i32, conn: DbConn) -> Result<Json<UserInfo>, Status> {
    use db::schema::users::dsl::*;
    use diesel::result::Error;
    debug!("get_by_id endpoint called");

    debug!("retrieving user");
    let user = users
        .find(user_id)
        .first::<User>(&*conn)
        .map_err(|e| match e {
            Error::NotFound => {
                debug!("user not found");
                Status::NotFound
            }
            _ => log_err(e),
        })?;

    Ok(Json(user.into()))
}

/// Log an error with Error priority, returning a `Status::InternalServiceError`.
fn log_err<T: Debug>(e: T) -> Status {
    error!("Encountered error -- {:?}", e);
    Status::InternalServerError
}

/// Hash and salt a password.
fn hash_password(user: &mut NewUser) -> Result<(), bcrypt::BcryptError> {
    debug!("hashing password");
    user.password = bcrypt::hash(&user.password, DEFAULT_COST)?;
    Ok(())
}

// Todo: secret key
/// Create an auth token containing a user's account details.
fn issue_token(user_info: &UserInfo) -> jwt::errors::Result<String> {
    debug!("creating token");
    jwt::encode(&Header::default(), user_info, "secret".as_ref())
}
