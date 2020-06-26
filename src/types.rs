use chrono::{DateTime, NaiveDate, Utc};
use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;

use crate::error;

pub type Result<T> = std::result::Result<T, error::Error>;
pub type Id = uuid::Uuid;
pub type PgPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type Date = NaiveDate;
pub type Time = DateTime<Utc>;

