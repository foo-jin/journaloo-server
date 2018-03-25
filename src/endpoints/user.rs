use std::env;

use bcrypt::{self, DEFAULT_COST};
use diesel;
use diesel::prelude::*;
use jwt::{self, Header};
use rand::Rng;
use rand::os::OsRng;
use rocket::http::Status;
use rocket::response::status;
use rocket_contrib::Json;
use sendgrid::mail::Mail;
use sendgrid::sg_client::SGClient;

use super::{log_db_err, log_err, ErrStatus};
use db::DbConn;
use db::models::user::{self, NewUser, User, UserInfo};
use endpoints::{PageQuery, PAGE_SIZE};

/// Registers a new user.
/// If the username or email is taken, fails with a `BadRequest` status.
/// If an unexpected error occurs, fails with an `InternalServiceError` status.
#[post("/user", format = "application/json", data = "<user>")]
pub fn signup(user: NewUser, conn: DbConn) -> Result<status::Created<String>, ErrStatus> {
    use db::schema::users;
    use diesel::result::Error;

    match users::table
        .filter(users::username.eq(&user.username))
        .or_filter(users::email.eq(&user.email))
        .first::<User>(&*conn)
    {
        Err(Error::NotFound) => (),
        Ok(_v) => return Err(status::Custom(Status::BadRequest, ())),
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
    updated_user: NewUser,
    conn: DbConn,
) -> Result<String, ErrStatus> {
    let user_info = user::update(&old_user, &updated_user, &*conn).map_err(log_db_err)?;
    let token = issue_token(&user_info).map_err(log_err)?;

    Ok(token)
}

/// Deletes a user, along with all its journeys and entries.
/// If an unexpected errors occur, fails with an `InternalServiceError` status.
#[delete("/user")]
pub fn delete(user: UserInfo, conn: DbConn) -> Result<(), ErrStatus> {
    user::delete(user, &*conn).map_err(log_db_err)
}

/// Login details of a user
#[derive(Deserialize)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}

/// Grants an auth token to a user if the credentials match.
/// If the user does not exist, fails with a `NotFound` status.
/// If the credentials do not match, fails with an `Unauthorized` status.
/// If an unexpected errors occur, fails with an `InternalServiceError` status.
#[post("/user/login", format = "application/json", data = "<user_login>")]
pub fn login(user_login: Json<UserLogin>, conn: DbConn) -> Result<String, ErrStatus> {
    use db::schema::users;

    let user = users::table
        .filter(users::username.eq(&user_login.username))
        .first::<User>(&*conn)
        .map_err(log_db_err)?;

    if !bcrypt::verify(&user_login.password, &user.password).map_err(log_err)? {
        debug!("couldn't verify password");
        return Err(status::Custom(Status::Unauthorized, ()));
    }

    let token = issue_token(&user.into()).map_err(log_err)?;

    Ok(token)
}

/// Reset a user's password.
/// Only confirmed to work with gmail accounts.
/// If the email does not belong to an existing user, fail with `NotFound` status.
/// If an unexpected errors occur, fails with an `InternalServiceError` status.
#[put("/user/<email_address>/reset")]
pub fn reset_password(
    email_address: String,
    conn: DbConn,
) -> Result<status::Accepted<()>, ErrStatus> {
    use db::schema::users;

    const RESET_DURATION: u32 = 5;

    lazy_static! {
        static ref API_KEY: String = {
            env::var("SENDGRID_API_KEY").expect("SENDGRID_API_KEY must be set")
        };
    }

    let user = users::table
        .filter(users::email.eq(&email_address))
        .first::<User>(&*conn)
        .map_err(log_db_err)?;

    let mut rand = OsRng::new().map_err(log_err)?;
    let mut new_pass = rand.gen_ascii_chars().take(50).collect::<String>();
    new_pass = bcrypt::hash(&new_pass, DEFAULT_COST).map_err(log_err)?;

    let mut email = Mail::new();
    email.add_to(email_address);
    email.add_from("password@journaloo.com");
    email.add_from_name("journaloo dev team");
    email.add_subject("Password reset");
    email.add_text(format!(
        "Your password has been reset. You can use the following code to log in during the next \
         {} hours. After that you will have to request another password reset.\n Code: {}",
        RESET_DURATION, new_pass
    ));

    SGClient::new(API_KEY.clone()).send(email).map_err(log_err)?;

    diesel::update(users::table.find(user.id))
        .set(users::password.eq(new_pass))
        .execute(&*conn)
        .map_err(log_db_err)?;

    Ok(status::Accepted(None))
}

/// Get a user by user ID.
/// If the user does not exist, fails with a `NotFound` status.
/// If an unexpected errors occur, fails with an `InternalServiceError` status.
#[get("/user/<user_id>")]
pub fn get_by_id(user_id: i32, conn: DbConn) -> Result<Json<UserInfo>, ErrStatus> {
    use db::schema::users;

    let user = users::table
        .find(user_id)
        .first::<User>(&*conn)
        .map_err(log_db_err)?;

    Ok(Json(user.into()))
}

// Note: `offset` usage here has bad performance on large page numbers
/// Gets a page of global users.
/// If an unexpected error occurs, fails with an `InternalServiceError` status.
#[get("/user/all?<query>")]
pub fn get_all(query: PageQuery, conn: DbConn) -> Result<Json<Vec<UserInfo>>, ErrStatus> {
    use db::schema::users;
    let page = query.page.0;

    let result = users::table
        .offset(page * PAGE_SIZE)
        .limit(PAGE_SIZE)
        .get_results::<User>(&*conn)
        .map_err(log_db_err)?;

    let result = result.into_iter().map(Into::into).collect();

    Ok(Json(result))
}

/// Create an auth token containing a user's account details.
fn issue_token(user_info: &UserInfo) -> jwt::errors::Result<String> {
    use SECRET;
    debug!("creating token");
    jwt::encode(&Header::default(), user_info, SECRET.as_bytes())
}
