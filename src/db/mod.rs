use diesel::pg::PgConnection;
use r2d2;
use r2d2_diesel::ConnectionManager;
use rocket::http::Status;
use rocket::Outcome;
use rocket::request::{self, FromRequest};
use rocket::Request;
use rocket::State;
use std::ops::Deref;
use std::env;
use dotenv::dotenv;

pub mod schema;
pub mod models;

// Alias to the type for a pool of Diesel PostgreSQL connections.
pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

// initializes a data pool
pub fn init_pool() -> Pool {
    dotenv().ok();

    // We need to make sure our database_url is set in our `.env` file. This will point to
    // our Postgres database.  If none is supplied, the program will error.
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(db_url);

    r2d2::Pool::builder()
        .max_size(5)
        .build(manager)
        .expect("failed to initialize db pool")
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
#[allow(non_upper_case_globals)]
/// Creates a test database connection
fn get_test_conn() -> DbConn {
    use diesel::Connection;
    use env_logger;

    let _ = env_logger::try_init();

    lazy_static! {
        static ref test_pool: Pool = init_pool();
    }

    let conn = test_pool.get().expect("failed to get db connection");
    conn.begin_test_transaction()
        .expect("failed to initialize test transaction");

    DbConn(conn)
}
