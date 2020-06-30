#![allow(warnings)]
#[macro_use]
extern crate diesel;

use std::borrow::Borrow;
use std::env;
use std::io::Write;
use std::ops::{Add, Div, Mul, Neg, Sub};
use std::str::FromStr;
use std::time::SystemTime;

use bigdecimal::BigDecimal;
use chrono::{Datelike, DateTime, Duration, NaiveDate, TimeZone, Utc};
use diesel::{deserialize::*, deserialize, Queryable, QueryableByName, r2d2, serialize};
pub use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::{Text, Varchar};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use strum;
use strum::*;
use strum_macros::{Display, EnumString};
use uuid;
use uuid::Uuid;

pub use error::{Error, Kind};

use crate::loan::Loan;
use crate::types::PgPool;

pub mod schema;
pub mod error;
pub mod account;
pub mod user;
pub mod bank_transaction;
pub mod account_transaction;
pub mod vault;
pub mod loan;
pub mod bank;
pub mod types;


/// Connect to PostgreSQL database
pub fn get_db_connection() -> PgPool {
	dotenv().ok();
	
	let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
	
	let manager = ConnectionManager::<PgConnection>::new(&database_url);
	let pool = r2d2::Pool::builder().build(manager)
		.expect("Failed to create pool.");
	
	pool
}

// pub mod prelude {
// 	pub use crate::account::{Account, NewAccount};
// 	pub use crate::account_transaction::{AccountTransaction, NewAccountTransaction};
// 	pub use crate::bank_transaction::{BankTransaction, BankTransactionType, NewBankTransaction};
// 	pub use crate::error::{Error, Kind};
// 	pub use crate::types::*;
// 	pub use crate::user::{NewUser, User};
// 	pub use crate::vault::{NewVault, Vault};
// }




