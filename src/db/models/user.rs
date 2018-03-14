use chrono::NaiveDateTime;
use db::schema::users;
use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use jwt::{decode, Validation};
use rocket::http::Status;
use rocket::Outcome;
use rocket::request::{self, FromRequest};
use rocket::Request;

#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub date: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub date: NaiveDateTime,
}

/// Create user record in database
pub fn create(conn: &PgConnection, user: &NewUser) -> diesel::QueryResult<UserInfo> {
    use db::schema::users;

    diesel::insert_into(users::table)
        .values(user)
        .get_result::<User>(conn)
        .map(|user| {
            info!("Created user \"{}\"", user.username);
            user.into()
        })
        .map_err(|e| {
            error!("Database error on user creation: {:?}", e);
            e
        })
}

impl<'a, 'r> FromRequest<'a, 'r> for UserInfo {
    type Error = ();

    /// Request guard for user authentication
    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let token = match request.headers().get_one("Authorization") {
            Some(jwt) => jwt,
            None => {
                debug!("Unauthorized request -- no token present: {}", request);
                return Outcome::Failure((Status::Unauthorized, ()));
            }
        };

        // Todo: secret key
        // Todo: error matching
        let token = match decode::<UserInfo>(token, b"secret", &Validation::default()) {
            Ok(token) => token,
            Err(_) => {
                debug!("Unauthorized request -- invalid token: {}", request);
                return Outcome::Failure((Status::Unauthorized, ()));
            }
        };

        debug!("Authorized request, username = {}", token.claims.username);
        Outcome::Success(token.claims)
    }
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        let User {
            id,
            username,
            email,
            date,
            ..
        } = user;

        UserInfo {
            id,
            username,
            email,
            date,
        }
    }
}

#[cfg(test)]
mod tests {
    use db::get_test_conn;
    use diesel::prelude::*;
    use diesel::query_builder;
    use dotenv::dotenv;
    use env_logger;
    use std::env;
    use super::*;

    #[test]
    fn create_user() {
        use super::users::dsl::*;
        let _ = env_logger::try_init();

        let conn = get_test_conn();
        let new_user = NewUser {
            username: "foo".to_string(),
            email: "foo@bar.com".to_string(),
            password: "asdf".to_string(),
        };

        let expected = create(&conn, &new_user).unwrap();
        let result = users
            .filter(username.eq(new_user.username))
            .first::<User>(&*conn)
            .expect("error getting result")
            .into();

        assert_eq!(expected, result);
    }
}
