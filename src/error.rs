use std::error;
use std::fmt;

// use diesel::r2d2::Error;
use diesel::result::DatabaseErrorKind::UniqueViolation;
use diesel::result::Error::{DatabaseError, NotFound};
use uuid::Error as uuidError;

// an error that can occur in this crate
#[derive(Debug)]
pub struct Error {
	kind: ErrorKind,
}

impl Error {
	pub(crate) fn new(kind: ErrorKind) -> Error {
		Error { kind }
	}
	
	pub fn kind(&self) -> &ErrorKind {
		&self.kind
	}
}

/// The kind of an error that can occur.
#[derive(Debug)]
pub enum ErrorKind {
	RecordAlreadyExists,
	RecordNotFound,
	DatabaseError(diesel::result::Error),
	OperationCanceled,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match &self.kind {
			ErrorKind::RecordAlreadyExists => write!(f, "this record violates a unique constraint"),
			ErrorKind::RecordNotFound => write!(f, "this record does not exist"),
			ErrorKind::DatabaseError(e) => write!(f, "database error: {:?}", e),
			ErrorKind::OperationCanceled => write!(f, "the running operating was cancelled")
		}
	}
}

impl From<diesel::result::Error> for Error {
	fn from(e: diesel::result::Error) -> Self {
		let kind = match e {
			DatabaseError(UniqueViolation, _) => ErrorKind::RecordAlreadyExists,
			NotFound => ErrorKind::RecordNotFound,
			
			// catch-all database error
			_ => ErrorKind::DatabaseError(e),
		};
		
		Self::new(kind)
	}
}

impl From<uuidError> for Error {
	fn from(e: uuidError) -> Self {
		Self::new(ErrorKind::RecordNotFound)
	}
}
