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

use lettre::EmailTransport;
use lettre::smtp::{ClientSecurity, ConnectionReuseParameters, SmtpTransportBuilder,
                   SUBMISSION_PORT};
use lettre::smtp::authentication::{Credentials, Mechanism};
use lettre::smtp::extension::ClientId;
use lettre_email::EmailBuilder;

use super::{log_db_err, log_err, ErrStatus};
use db::DbConn;
use db::models::user::{self, NewUser, User, UserInfo};

/// Registers a new user.
/// If the username or email is taken, fails with a `BadRequest` status.
/// If an unexpected error occurs, fails with an `InternalServiceError` status.
#[post("/user", format = "application/json", data = "<user>")]
pub fn signup(user: Json<NewUser>, conn: DbConn) -> Result<status::Created<String>, ErrStatus> {
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
            return Err(status::Custom(Status::BadRequest, ()));
        }
        Err(e) => return Err(log_err(e)),
    }

    let user_info = user::create(&user, &conn).map_err(log_err)?;
    let token = issue_token(&user_info).map_err(log_err)?;

    Ok(status::Created(String::new(), Some(token)))
}

/// Updates an existing user.
/// If the user does not exist, fails with a `NotFound` status.
/// If unexpected errors occur, fails with an `InternalServiceError` status.
#[put("/user", format = "application/json", data = "<updated_user>")]
pub fn update(
    old_user: UserInfo,
    updated_user: Json<NewUser>,
    conn: DbConn,
) -> Result<String, ErrStatus> {
    let mut updated_user = updated_user.into_inner();
    hash_password(&mut updated_user).map_err(log_err)?;

    let user_info = user::update(&old_user, &updated_user, &*conn).map_err(log_db_err)?;
    let token = issue_token(&user_info).map_err(log_err)?;

    Ok(token)
}

/// Deletes a user, along with all its journeys and entries.
/// If an unexpected errors occur, fails with an `InternalServiceError` status.
#[delete("/user")]
pub fn delete(user: UserInfo, conn: DbConn) -> Result<(), ErrStatus> {
    user::delete(user, &*conn).map_err(log_err)
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
    use db::schema::users::dsl::*;

    let user = users
        .filter(username.eq(&user_login.username))
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
/// If the email does not belong to an existing user, fail with `NotFound` status.
/// If an unexpected errors occur, fails with an `InternalServiceError` status.
#[put("/user/reset_password/<email_address>")]
#[allow(unused_variables)]
pub fn reset_password(
    email_address: String,
    conn: DbConn,
) -> Result<status::Accepted<()>, ErrStatus> {
    use db::schema::users::dsl::*;

    const RESET_DURATION: u32 = 5;

    lazy_static! {
        static ref MAILGUN_NAME: String = {
            env::var("MAILGUN_NAME").expect("MAILGUN_NAME must be set")
        };
        static ref MAILGUN_DOMAIN: String = {
            env::var("MAILGUN_DOMAIN").expect("MAILGUN_DOMAIN must be set")
        };
        static ref MAILGUN_PASSWORD: String = {
            env::var("MAILGUN_PASSWORD").expect("MAILGUN_PASSWORD must be set")
        };
    }

    let user = users
        .filter(email.eq(&email_address))
        .first::<User>(&*conn)
        .map_err(log_db_err)?;

    let mut rand = OsRng::new().map_err(log_err)?;
    let mut new_pass = rand.gen_ascii_chars().take(50).collect::<String>();
    new_pass = bcrypt::hash(&new_pass, DEFAULT_COST).map_err(log_err)?;

    let send_email = EmailBuilder::new()
        .to(email_address)
        .from("traveloo@example.com")
        .subject("Password reset")
        .text(format!(
            "Your password has been reset. You can use the following code to log in during the \
             next {} hours. After that you will have to request another password reset.\n Code: {}",
            RESET_DURATION, new_pass
        ))
        .build()
        .unwrap();

    let mut mailer =
        SmtpTransportBuilder::new(("smtp.mailgun.org", SUBMISSION_PORT), ClientSecurity::None)
            .map_err(log_err)?
            .hello_name(ClientId::new(MAILGUN_DOMAIN.to_string()))
            .credentials(Credentials::new(
                MAILGUN_NAME.to_string(),
                MAILGUN_PASSWORD.to_string(),
            ))
            .smtp_utf8(true)
            .authentication_mechanism(Mechanism::Plain)
            .connection_reuse(ConnectionReuseParameters::ReuseUnlimited)
            .build();

    mailer.send(&send_email).map_err(log_err)?;

    diesel::update(users.find(user.id))
        .set(password.eq(new_pass))
        .execute(&*conn)
        .map_err(log_db_err)?;

    Ok(status::Accepted(None))
}

/// Get a user by user ID.
/// If the user does not exist, fails with a `NotFound` status.
/// If an unexpected errors occur, fails with an `InternalServiceError` status.
#[get("/user/<user_id>")]
pub fn get_by_id(user_id: i32, conn: DbConn) -> Result<Json<UserInfo>, ErrStatus> {
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
