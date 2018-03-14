use diesel::pg::PgConnection;
use r2d2;
use r2d2_diesel::ConnectionManager;
use rocket::http::Status;
use rocket::Outcome;
use rocket::request::{self, FromRequest};
use rocket::Request;
use rocket::State;
use std::ops::Deref;

pub mod schema;
pub mod models;

// Alias to the type for a pool of Diesel PostgreSQL connections.
pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

// initializes a data pool
pub fn init_pool(db_url: String) -> Pool {
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    r2d2::Pool::new(manager).expect("failed to initialize db pool")
}

// Connection request guard: a wrapper around an r2d2 pooled connection.
pub struct DbConn(pub r2d2::PooledConnection<ConnectionManager<PgConnection>>);

/// Attempts to retrieve a single connection from the managed database pool. If
/// no pool is currently managed, fails with an `InternalServerError` status. If
/// no connections are available, fails with a `ServiceUnavailable` status.
impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<DbConn, ()> {
        let pool = request.guard::<State<Pool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

// For the convenience of using an &DbConn as an &PgConnection.
impl Deref for DbConn {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
pub fn create_test_connection() -> PgConnection {
    use dotenv::dotenv;
    use std::env;

    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let conn = PgConnection::establish(&database_url).expect("failed to establish db connection");
    conn.begin_test_transaction().expect("failed to initialize test transaction");
    conn
}