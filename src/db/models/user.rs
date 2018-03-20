use chrono::NaiveDateTime;
use db::schema::users;
use diesel;
use diesel::prelude::*;
use jwt::{decode, Validation};
use rocket::Outcome;
use rocket::Request;
use rocket::http::Status;
use rocket::request::{self, FromRequest};

#[derive(Queryable, Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub date: NaiveDateTime,
}

#[derive(Insertable, AsChangeset, Deserialize)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

type UpdateUser = NewUser;

#[derive(Identifiable, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[table_name = "users"]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
    pub email: String,
}

/// Creates a user record in the database
pub fn create(user: &NewUser, conn: &PgConnection) -> diesel::QueryResult<UserInfo> {
    use db::schema::users::dsl::*;
    debug!("creating user record in db");

    diesel::insert_into(users)
        .values(user)
        .get_result::<User>(conn)
        .map(|user| {
            debug!("Created user {:?}", user);
            user.into()
        })
        .map_err(|e| {
            error!("Failed to create user -- {:?}", e);
            e
        })
}

/// Updates a user record in the database
pub fn update(
    old_user: &UserInfo,
    user: &UpdateUser,
    conn: &PgConnection,
) -> diesel::QueryResult<UserInfo> {
    use db::schema::users::dsl::*;

    let target = users.find(old_user.id);
    diesel::update(target)
        .set(user)
        .get_result::<User>(conn)
        .map(|user| {
            debug!("Updated user\n{:?}\nto {:?})", old_user, user);
            user.into()
        })
        .map_err(|e| {
            error!("Failed to update user -- {:?}", e);
            e
        })
}

/// Deletes a user, and its owned journeys and entries
pub fn delete(user: UserInfo, conn: &PgConnection) -> diesel::QueryResult<()> {
    use db::models::entry::Entry;
    use db::models::journey::Journey;
    use db::schema::journeys::dsl::*;
    use db::schema::users::dsl::*;

    let mut del_journeys = 0;
    let mut del_entries = 0;

    for journey in Journey::belonging_to(&user).load::<Journey>(&*conn)? {
        del_entries += diesel::delete(Entry::belonging_to(&journey)).execute(&*conn)?;

        del_journeys += diesel::delete(journeys.find(journey.id)).execute(&*conn)?;
    }

    let target = users.find(user.id);
    let del_users = diesel::delete(target).execute(&*conn)?;

    debug!(
        "Deleted {} users, {} journeys, and {} entries",
        del_users, del_journeys, del_entries
    );

    Ok(())
}

impl<'a, 'r> FromRequest<'a, 'r> for UserInfo {
    type Error = ();

    /// Request guard for user authentication
    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        use SECRET;
        debug!("verifying auth token");

        let token = match request.headers().get_one("Authorization") {
            Some(jwt) => jwt,
            None => {
                debug!("Unauthorized request -- no token present: {}", request);
                return Outcome::Failure((Status::Unauthorized, ()));
            }
        };

        let token = match decode::<UserInfo>(token, SECRET.as_bytes(), &Validation::default()) {
            Ok(token) => token,
            Err(e) => {
                debug!("Unauthorized request -- {:?}: {}", e, request);
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
            ..
        } = user;

        UserInfo {
            id,
            username,
            email,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use db::get_test_conn;

    #[test]
    fn create_user() {
        use super::users::dsl::*;
        let conn = get_test_conn();

        let new_user = NewUser {
            username: "foo".to_string(),
            email: "foo@bar.com".to_string(),
            password: "asdf".to_string(),
        };

        let expected = create(&new_user, &conn).expect("failed to create user");
        let result = users
            .filter(username.eq(new_user.username))
            .first::<User>(&*conn)
            .expect("error getting result")
            .into();

        assert_eq!(expected, result);
    }

    #[test]
    fn update_user() {
        use super::users::dsl::*;
        let conn = get_test_conn();

        let mut new_user = NewUser {
            username: "foo".to_string(),
            email: "foo@bar.com".to_string(),
            password: "asdf".to_string(),
        };

        let user = create(&new_user, &conn).expect("failed to create user");

        new_user.username = "bar".to_string();
        let expected = update(&user, &new_user, &*conn).expect("failed to update user");
        let result = users
            .filter(username.eq(new_user.username))
            .first::<User>(&*conn)
            .expect("error getting result")
            .into();

        assert_eq!(expected, result);
    }

    #[test]
    fn delete_user() {
        use super::users::dsl::*;
        use diesel::NotFound;
        let conn = get_test_conn();

        let new_user = NewUser {
            username: "foo".to_string(),
            email: "foo@bar.com".to_string(),
            password: "asdf".to_string(),
        };

        let user = create(&new_user, &conn).expect("failed to create user");
        let uid = user.id;
        delete(user, &conn).expect("failed to delete user");

        match users.find(uid).first::<User>(&*conn) {
            Err(NotFound) => (),
            Ok(_user) => panic!("user not deleted"),
            Err(e) => panic!("failed to delete user -- {:?}", e),
        }
    }
}
