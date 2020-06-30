use std::borrow::Borrow;
use std::ops::Neg;
use std::time::SystemTime;

use bigdecimal::BigDecimal;
use diesel::{
	associations,
	deserialize,
	pg::Pg,
	PgConnection,
	prelude::*,
	serialize,
	sql_types::Varchar,
};

use crate::PgPool;
use crate::schema::accounts;
use crate::types::{Result, Time};

#[derive(Queryable, Identifiable, PartialEq, Debug)]
pub struct Account {
	pub id: uuid::Uuid,
	pub user_id: uuid::Uuid,
	pub account_type: AccountType,
	pub amount: BigDecimal,
	pub created_at: Time,
	pub is_open: bool,
}

#[derive(Insertable)]
#[table_name = "accounts"]
pub struct NewAccount {
	pub user_id: uuid::Uuid,
	pub account_type: AccountType,
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

impl serialize::ToSql<Varchar, Pg> for AccountType {
	fn to_sql<W: std::io::Write>(&self, out: &mut serialize::Output<W, Pg>) -> serialize::Result {
		serialize::ToSql::<Varchar, Pg>::to_sql(self.as_str(), out)
	}
}

impl deserialize::FromSql<Varchar, Pg> for AccountType {
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

pub struct Repo {
	db: PgPool,
}

impl Repo {
	pub fn new(db: PgPool) -> Self {
		Repo { db }
	}
	
	pub fn create_account(&self, new_account: NewAccount) -> Result<Account> {
		let conn = &self.db.get()?;
		diesel::insert_into(accounts::table)
			.values(&new_account)
			.get_result(conn)
			.map_err(Into::into)
	}
	
	pub fn find_accounts(&self, user_id: &uuid::Uuid) -> Result<Vec<Account>> {
		let conn = &self.db.get()?;
		accounts::table
			.filter(accounts::user_id.eq(user_id))
			.select((accounts::all_columns))
			.load::<Account>(conn)
			.map_err(Into::into)
	}
	
	pub fn find_by_id(&self, account_id: &uuid::Uuid) -> Result<Account> {
		let conn = &self.db.get()?;
		accounts::table
			.filter(accounts::id.eq(account_id))
			.select((accounts::all_columns))
			.first::<Account>(conn)
			.map_err(Into::into)
	}
	
	pub fn increment(&self, account_id: &uuid::Uuid, amount: &BigDecimal) -> Result<Account> {
		self.transact(account_id, amount)
	}
	
	pub fn decrement(&self, account_id: &uuid::Uuid, amount: &BigDecimal) -> Result<Account> {
		let neg = amount.neg();
		self.transact(account_id, &neg)
	}
	
	fn transact(&self, account_id: &uuid::Uuid, amount: &BigDecimal) -> Result<Account> {
		let conn = &self.db.get()?;
		diesel::update(accounts::table)
			.filter(accounts::id.eq(account_id))
			.set(accounts::amount.eq(accounts::amount + amount))
			.get_result(conn)
			.map_err(Into::into)
	}
}

