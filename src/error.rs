use std::error;
use std::fmt;

use diesel::result::DatabaseErrorKind::UniqueViolation;
use diesel::result::Error::{DatabaseError, NotFound};
use r2d2;
use uuid::Error as uuidError;

use crate::account;

// an error that can occur in this crate
#[derive(Debug, PartialEq)]
pub struct Error {
	kind: Kind,
}

impl Error {
	pub fn new(kind: Kind) -> Error {
		Error { kind }
	}
	
	pub fn kind(&self) -> &Kind {
		&self.kind
	}
}

/// The kind of an error that can occur.
#[derive(Debug, PartialEq)]
pub enum Kind {
	RecordAlreadyExists,
	RecordNotFound,
	DatabaseError(diesel::result::Error),
	InadequateFunds,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match &self.kind {
			Kind::RecordAlreadyExists => write!(f, "this record violates a unique constraint"),
			Kind::RecordNotFound => write!(f, "this record does not exist"),
			// Kind::DatabaseError(e) => write!(f, "database error: {:?}", e),
			Kind::DatabaseError(e) => write!(f, "database error: {:?}", e),
			Kind::InadequateFunds => write!(f, "not enough funds in account")
		}
	}
}


impl From<diesel::result::Error> for Error {
	fn from(e: diesel::result::Error) -> Self {
		let kind = match e {
			DatabaseError(UniqueViolation, _) => Kind::RecordAlreadyExists,
			NotFound => Kind::RecordNotFound,
			
			// catch-all database error
			_ => Kind::DatabaseError(e),
		};
		
		Self::new(kind)
	}
}

impl From<r2d2::Error> for Error {
	fn from(e: r2d2::Error) -> Self {
		let kind = match e {
			//todo: fix this
			_ => Kind::RecordNotFound,
		};
		
		Self::new(kind)
	}
}

