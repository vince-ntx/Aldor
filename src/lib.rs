#![allow(warnings)]
#[macro_use]
extern crate diesel;

use std::borrow::Borrow;
use std::env;
use std::error::Error;
use std::io::Write;
use std::ops::{Add, Neg};
use std::str::FromStr;
use std::time::SystemTime;

use bigdecimal::BigDecimal;
use diesel::{deserialize::*, deserialize, Queryable, QueryableByName, serialize};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::{Text, Varchar};
use serde::{Deserialize, Serialize};
use uuid;
use uuid::Uuid;

use dotenv::dotenv;
use schema::*;

pub mod schema;
mod error;

/// Connect to PostgreSQL database
pub fn get_db_connection() -> PgConnection {
	dotenv().ok();
	
	let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
	
	PgConnection::establish(&database_url).expect(&format!("error connecting to {}", database_url))
}

type Result<T> = std::result::Result<T, error::Error>;

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

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
	pub email: &'a str,
	pub first_name: &'a str,
	pub family_name: &'a str,
	pub phone_number: Option<&'a str>,
}

pub struct UserRepo<'a> {
	db: &'a PgConnection,
}

impl<'a> UserRepo<'a> {
	pub fn new(db: &'a PgConnection) -> Self {
		UserRepo { db }
	}
	
	pub fn create_user(&self, new_user: NewUser) -> Result<User> {
		use schema::users::dsl::*;
		diesel::insert_into(users)
			.values(&new_user)
			.get_result(self.db)
			.map_err(Into::into)
	}
	
	pub fn find_user(&self, key: UserKey<'a>) -> Result<User> {
		match key {
			UserKey::ID(id) => {
				users::table
					.find(id)
					.first::<User>(self.db)
					.map_err(Into::into)
			}
			UserKey::Email(email) => {
				users::table
					.filter(users::email.eq(email))
					.first::<User>(self.db)
					.map_err(Into::into)
			}
		}
	}
}

pub enum UserKey<'a> {
	ID(uuid::Uuid),
	Email(&'a str),
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


#[derive(Insertable)]
#[table_name = "accounts"]
pub struct NewAccount {
	pub user_id: uuid::Uuid,
	pub account_type: AccountType,
	pub amount: BigDecimal,
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

pub struct AccountRepo<'a> {
	db: &'a PgConnection,
}

impl<'a> AccountRepo<'a> {
	pub fn new(db: &'a PgConnection) -> Self {
		AccountRepo { db }
	}
	
	pub fn create_account(&self, new_account: NewAccount) -> Result<Account> {
		diesel::insert_into(accounts::table)
			.values(&new_account)
			.get_result(self.db)
			.map_err(Into::into)
	}
	
	pub fn find_accounts(&self, user_id: &uuid::Uuid) -> Result<Vec<Account>> {
		accounts::table
			.filter(accounts::user_id.eq(user_id))
			.select((accounts::all_columns))
			.load::<Account>(self.db)
			.map_err(Into::into)
	}
	
	pub fn find_account(&self, account_id: &uuid::Uuid) -> Result<Account> {
		accounts::table
			.filter(accounts::id.eq(account_id))
			.select((accounts::all_columns))
			.first::<Account>(self.db)
			.map_err(Into::into)
	}
	
	pub fn transact(&self, k: TransactionKey, account_id: &uuid::Uuid, value: &BigDecimal) ->
	Result<Account> {
		let neg_value = value.neg();
		let v = match k {
			TransactionKey::Deposit => value,
			TransactionKey::Withdraw => &neg_value,
		};
		
		diesel::update(accounts::table)
			.filter(accounts::id.eq(account_id))
			.set(accounts::amount.eq(accounts::amount + v))
			.get_result(self.db)
			.map_err(Into::into)
	}
}

#[derive(Serialize, Debug, AsExpression, FromSqlRow, PartialEq)]
#[sql_type = "Varchar"]
pub enum TransactionKey {
	Deposit,
	Withdraw,
}

impl TransactionKey {
	pub fn as_str(&self) -> &str {
		match self {
			TransactionKey::Deposit => "deposit",
			TransactionKey::Withdraw => "withdraw",
		}
	}
}


impl ToSql<Varchar, Pg> for TransactionKey {
	fn to_sql<W: std::io::Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
		ToSql::<Varchar, Pg>::to_sql(self.as_str(), out)
	}
}

impl FromSql<Varchar, Pg> for TransactionKey {
	fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
		let o = bytes.ok_or_else(|| "error deserializing from varchar")?;
		let x = std::str::from_utf8(o)?;
		match x {
			"deposit" => Ok(TransactionKey::Deposit),
			"withdraw" => Ok(TransactionKey::Withdraw),
			_ => Err("invalid transaction key".into())
		}
	}
}

#[derive(Insertable)]
#[table_name = "transactions"]
pub struct NewTransaction {
	pub from_id: Option<uuid::Uuid>,
	pub to_id: Option<uuid::Uuid>,
	pub transaction_type: TransactionKey,
	pub amount: BigDecimal,
}

#[derive(Queryable, Identifiable, PartialEq, Debug)]
pub struct Transaction {
	pub id: uuid::Uuid,
	pub from_id: Option<uuid::Uuid>,
	pub to_id: Option<uuid::Uuid>,
	pub transaction_type: TransactionKey,
	pub amount: BigDecimal,
	pub timestamp: SystemTime,
}

pub struct TransactionRepo<'a> {
	db: &'a PgConnection,
}

impl<'a> TransactionRepo<'a> {
	pub fn new(db: &'a PgConnection) -> Self {
		TransactionRepo { db }
	}
	
	pub fn create(&self, new_transaction: NewTransaction) -> Result<Transaction> {
		diesel::insert_into(transactions::table)
			.values(&new_transaction)
			.get_result::<Transaction>(self.db)
			.map_err(Into::into)
	}
}



