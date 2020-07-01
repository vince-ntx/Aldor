use std::fmt;

use diesel::result::DatabaseErrorKind::UniqueViolation;
use diesel::result::Error::{DatabaseError, NotFound};
use r2d2;
use uuid::Error as uuidError;

use crate::error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
	RecordAlreadyExists,
	RecordNotFound,
	DatabaseError(diesel::result::Error),
	Connection(r2d2::Error),
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
		Error::Connection(e)
	}
}
