#![allow(warnings)]
#[macro_use]
extern crate diesel;

use std::borrow::Borrow;
use std::env;
use std::io::Write;
use std::ops::{Add, Neg};
use std::str::FromStr;
use std::time::SystemTime;

use bigdecimal::BigDecimal;
use diesel::{deserialize::*, deserialize, Queryable, QueryableByName, r2d2, serialize};
pub use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::{Text, Varchar};
use serde::{Deserialize, Serialize};
use uuid;
use uuid::Uuid;

use dotenv::dotenv;
use schema::*;

pub use crate::account::*;
pub use crate::error::*;
pub use crate::schema::*;
pub use crate::transaction::*;
pub use crate::user::*;

mod schema;
pub mod error;
pub mod account;
pub mod user;
pub mod transaction;

type Result<T> = std::result::Result<T, error::Error>;
pub type PgPool = r2d2::Pool<ConnectionManager<PgConnection>>;

/// Connect to PostgreSQL database
pub fn get_db_connection() -> PgPool {
	dotenv().ok();
	
	let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
	
	let manager = ConnectionManager::<PgConnection>::new(&database_url);
	let pool = r2d2::Pool::builder().build(manager)
		.expect("Failed to create pool.");
	
	pool
}

pub struct NewBankService<'a> {
	pub db: PgPool,
	pub user_repo: &'a user::Repo,
	pub account_repo: &'a account::Repo,
	pub transaction_repo: &'a transaction::Repo,
}

pub struct BankService<'a> {
	db: PgPool,
	//todo: abstract this out into a trait
	user_repo: &'a user::Repo,
	account_repo: &'a account::Repo,
	transaction_repo: &'a transaction::Repo,
}

impl<'a> BankService<'a> {
	pub fn new(n: NewBankService<'a>) -> Self {
		BankService {
			db: n.db,
			user_repo: n.user_repo,
			account_repo: n.account_repo,
			transaction_repo: n.transaction_repo,
		}
	}
	pub fn deposit(&self, account_id: &uuid::Uuid, amount: &BigDecimal) -> Result<Account> { //todo:: add result
		let conn = &self.db.get()?;
		conn.transaction::<Account, error::Error, _>(|| {
			self.transaction_repo.create(transaction::NewTransaction {
				from_id: None,
				to_id: Some(account_id.clone()),
				transaction_type: TransactionType::Deposit,
				amount: amount.to_owned(),
			})?;
			
			let account = self.account_repo.transact(TransactionType::Deposit, account_id, amount)?;
			
			Ok(account)
		})
	}
	
	pub fn withdraw(&self, account_id: &uuid::Uuid, amount: &BigDecimal) -> Result<Account> {
		let mut account = self.account_repo.find_account(account_id).expect("get account");
		if account.amount.lt(amount) {
			return Err(Error::new(Kind::InadequateFunds));
		}
		
		let conn = &self.db.get()?;
		conn.transaction::<(), error::Error, _>(|| {
			self.transaction_repo.create(transaction::NewTransaction {
				from_id: Some(account_id.clone()),
				to_id: None,
				transaction_type: TransactionType::Withdraw,
				amount: amount.to_owned(),
			})?;
			
			account = self.account_repo.transact(TransactionType::Withdraw, account_id, amount)?;
			
			Ok(())
		});
		
		Ok(account)
	}
}

#[derive(Queryable, Identifiable, PartialEq, Debug)]
pub struct User {
	pub id: uuid::Uuid,
	pub email: String,
	pub first_name: String,
	pub family_name: String,
	pub phone_number: Option<String>,
	/* TODO: add additional info here including
	- date of birth
	- home address
	 */
}


#[derive(Queryable, Identifiable, Associations, PartialEq, Debug)]
#[belongs_to(User)]
pub struct Account {
	pub id: uuid::Uuid,
	pub user_id: uuid::Uuid,
	pub account_type: AccountType,
	pub amount: BigDecimal,
	pub created_at: SystemTime,
	pub is_open: bool,
}

#[derive(AsExpression, FromSqlRow, PartialEq, Debug)]
#[sql_type = "Varchar"]
pub enum AccountType {
	Checking,
	Savings,
}

impl AccountType {
	pub fn as_str(&self) -> &str {
		match self {
			AccountType::Checking => "checking",
			AccountType::Savings => "savings",
		}
	}
}

impl ToSql<Varchar, Pg> for AccountType {
	fn to_sql<W: std::io::Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
		ToSql::<Varchar, Pg>::to_sql(self.as_str(), out)
	}
}

impl FromSql<Varchar, Pg> for AccountType {
	fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
		let o = bytes.ok_or_else(|| "error deserializing from varchar")?;
		let x = std::str::from_utf8(o)?;
		match x {
			"checking" => Ok(AccountType::Checking),
			"savings" => Ok(AccountType::Savings),
			_ => Err("invalid account type".into())
		}
	}
}

#[derive(Queryable, Identifiable, PartialEq, Debug)]
pub struct Transaction {
	pub id: uuid::Uuid,
	pub from_id: Option<uuid::Uuid>,
	pub to_id: Option<uuid::Uuid>,
	pub transaction_type: TransactionType,
	pub amount: BigDecimal,
	pub timestamp: SystemTime,
}

#[derive(Debug, AsExpression, FromSqlRow, PartialEq)]
#[sql_type = "Varchar"]
pub enum TransactionType {
	Deposit,
	Withdraw,
}

impl TransactionType {
	pub fn as_str(&self) -> &str {
		match self {
			TransactionType::Deposit => "deposit",
			TransactionType::Withdraw => "withdraw",
		}
	}
}

impl ToSql<Varchar, Pg> for TransactionType {
	fn to_sql<W: std::io::Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
		ToSql::<Varchar, Pg>::to_sql(self.as_str(), out)
	}
}

impl FromSql<Varchar, Pg> for TransactionType {
	fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
		let o = bytes.ok_or_else(|| "error deserializing from varchar")?;
		let x = std::str::from_utf8(o)?;
		match x {
			"deposit" => Ok(TransactionType::Deposit),
			"withdraw" => Ok(TransactionType::Withdraw),
			_ => Err("invalid transaction key".into())
		}
	}
}
