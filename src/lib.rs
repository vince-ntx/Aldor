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
use crate::account_transaction::NewAccountTransaction;
pub use crate::bank_transaction::*;
pub use crate::error::*;
pub use crate::schema::*;
pub use crate::user::*;

mod schema;
pub mod error;
pub mod account;
pub mod user;
pub mod bank_transaction;
pub mod account_transaction;
pub mod vault;

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
	pub vault_repo: &'a vault::Repo,
	pub account_repo: &'a account::Repo,
	pub bank_transaction_repo: &'a bank_transaction::Repo,
	pub account_transaction_repo: &'a account_transaction::Repo,
}

pub struct BankService<'a> {
	//todo: abstract this out into a trait
	db: PgPool,
	user_repo: &'a user::Repo,
	account_repo: &'a account::Repo,
	vault_repo: &'a vault::Repo,
	bank_transaction_repo: &'a bank_transaction::Repo,
	account_transaction_repo: &'a account_transaction::Repo,
}

impl<'a> BankService<'a> {
	pub fn new(n: NewBankService<'a>) -> Self {
		BankService {
			db: n.db,
			user_repo: n.user_repo,
			account_repo: n.account_repo,
			vault_repo: n.vault_repo,
			bank_transaction_repo: n.bank_transaction_repo,
			account_transaction_repo: n.account_transaction_repo,
		}
	}
	
	pub fn deposit(&self, account_id: &uuid::Uuid, vault_name: &str, amount: &BigDecimal) -> Result<Account> { //todo:: add result
		let conn = &self.db.get()?;
		conn.transaction::<Account, error::Error, _>(|| {
			self.bank_transaction_repo.create(bank_transaction::NewBankTransaction {
				account_id,
				vault_name,
				transaction_type: BankTransactionType::Deposit,
				amount,
			})?;
			
			let account = self.account_repo.increment(account_id, amount)?;
			self.vault_repo.transact(BankTransactionType::Deposit, vault_name, amount)?;
			
			Ok(account)
		})
	}
	
	pub fn withdraw(&self, account_id: &uuid::Uuid, vault_name: &str, amount: &BigDecimal) -> Result<Account> {
		let mut account = self.account_repo.find_account(account_id).expect("get account");
		if account.amount.lt(amount) {
			return Err(Error::new(Kind::InadequateFunds));
		}
		
		let conn = &self.db.get()?;
		conn.transaction::<(), error::Error, _>(|| {
			self.bank_transaction_repo.create(bank_transaction::NewBankTransaction {
				account_id,
				vault_name,
				transaction_type: BankTransactionType::Withdraw,
				amount,
			})?;
			
			account = self.account_repo.decrement(account_id, amount)?;
			self.vault_repo.transact(BankTransactionType::Withdraw, vault_name, amount)?;
			
			Ok(())
		});
		
		Ok(account)
	}
	
	pub fn send_funds(&self, sender_id: &uuid::Uuid, receiver_id: &uuid::Uuid, amount: &BigDecimal) -> Result<AccountTransaction> {
		let conn = &self.db.get()?;
		conn.transaction::<AccountTransaction, error::Error, _>(|| {
			let transaction = self.account_transaction_repo.transfer(NewAccountTransaction {
				sender_id,
				receiver_id,
				amount,
			})?;
			
			self.account_repo.increment(receiver_id, amount)?;
			self.account_repo.decrement(sender_id, amount)?;
			
			Ok(transaction)
		})
	}
}

#[derive(Queryable, PartialEq, Debug)]
pub struct Vault {
	pub name: String,
	pub amount: BigDecimal,
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
pub struct BankTransaction {
	pub id: uuid::Uuid,
	pub account_id: uuid::Uuid,
	pub vault_name: String,
	pub transaction_type: BankTransactionType,
	pub amount: BigDecimal,
	pub created_at: SystemTime,
}

#[derive(Debug, AsExpression, FromSqlRow, PartialEq)]
#[sql_type = "Varchar"]
pub enum BankTransactionType {
	Deposit,
	Withdraw,
}

impl BankTransactionType {
	pub fn as_str(&self) -> &str {
		match self {
			BankTransactionType::Deposit => "deposit",
			BankTransactionType::Withdraw => "withdraw",
		}
	}
}

impl ToSql<Varchar, Pg> for BankTransactionType {
	fn to_sql<W: std::io::Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
		ToSql::<Varchar, Pg>::to_sql(self.as_str(), out)
	}
}

impl FromSql<Varchar, Pg> for BankTransactionType {
	fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
		let o = bytes.ok_or_else(|| "error deserializing from varchar")?;
		let x = std::str::from_utf8(o)?;
		match x {
			"deposit" => Ok(BankTransactionType::Deposit),
			"withdraw" => Ok(BankTransactionType::Withdraw),
			_ => Err("invalid transaction key".into())
		}
	}
}

#[derive(Queryable, Identifiable, PartialEq, Debug)]
pub struct AccountTransaction {
	pub id: uuid::Uuid,
	pub sender_id: uuid::Uuid,
	pub receiver_id: uuid::Uuid,
	pub amount: BigDecimal,
	pub created_at: SystemTime,
}
