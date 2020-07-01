use std::error;
use std::fmt;

use crate::{account, db};

// an error that can occur in this crate
#[derive(Debug)]
pub struct Error {
	kind: ErrorKind,
}

impl Error {
	pub fn new(kind: ErrorKind) -> Error {
		Error { kind }
	}
	
	pub fn kind(&self) -> &ErrorKind {
		&self.kind
	}
}

/// The kind of an error that can occur.
#[derive(Debug)]
pub enum ErrorKind {
	Database(db::Error),
	InadequateFunds,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match &self.kind {
			ErrorKind::Database(e) => write!(f, "db error: {}", e),
			ErrorKind::InadequateFunds => write!(f, "not enough funds in account")
		}
	}
}

impl From<db::Error> for Error {
	fn from(e: db::Error) -> Self {
		Error::new(ErrorKind::Database(e))
	}
}

impl From<r2d2::Error> for Error {
	fn from(e: r2d2::Error) -> Self {
		Error::new(ErrorKind::Database(db::Error::from(e)))
	}
}

impl From<diesel::result::Error> for Error {
	fn from(e: diesel::result::Error) -> Self {
		Error::new(ErrorKind::Database(db::Error::from(e)))
	}
}




