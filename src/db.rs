use std::{env, fmt};

use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel::result::DatabaseErrorKind::UniqueViolation;
use diesel::result::Error::{DatabaseError, NotFound};
use dotenv::dotenv;
use r2d2;
use uuid::Error as uuidError;

pub type Result<T> = std::result::Result<T, Error>;
pub type PgPool = r2d2::Pool<ConnectionManager<PgConnection>>;

/// Connect to PostgreSQL database
pub fn pg_connection() -> PgPool {
	dotenv().ok();
	
	let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
	
	let manager = ConnectionManager::<PgConnection>::new(&database_url);
	let pool = r2d2::Pool::builder().build(manager)
		.expect("Failed to create pool.");
	
	pool
}

#[derive(Debug, PartialEq)]
pub enum Error {
	RecordAlreadyExists,
	RecordNotFound,
	DatabaseError(diesel::result::Error),
	Connection(String),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Error::RecordAlreadyExists => write!(f, "record violates a unique constraint"),
			Error::RecordNotFound => write!(f, "record does not exist"),
			Error::Connection(e) => write!(f, "opening database connection: {}", e),
			Error::DatabaseError(e) => write!(f, "database error: {:?}", e),
		}
	}
}

impl From<diesel::result::Error> for Error {
	fn from(e: diesel::result::Error) -> Self {
		let err = match e {
			DatabaseError(UniqueViolation, _) => Error::RecordAlreadyExists,
			NotFound => Error::RecordNotFound,
			
			_ => Error::DatabaseError(e),
		};
		
		err
	}
}

impl From<r2d2::Error> for Error {
	fn from(e: r2d2::Error) -> Self {
		Error::Connection(e.to_string())
	}
}
