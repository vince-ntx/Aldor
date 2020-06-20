use std::ops::Neg;
use std::time::SystemTime;

use bigdecimal::BigDecimal;
use diesel::{deserialize, PgConnection, serialize};
use diesel::deserialize::FromSql;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::Varchar;

use crate::Result;
use crate::schema::*;
use crate::transaction;
use crate::transaction::Type;
use crate::user;

#[derive(Queryable, Identifiable, Associations, PartialEq, Debug)]
#[belongs_to(user::User)]
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
	
	pub fn transact(&self, k: Type, account_id: &uuid::Uuid, value: &BigDecimal) ->
	Result<Account> {
		let neg_value = value.neg();
		let v = match k {
			Type::Deposit => value,
			Type::Withdraw => &neg_value,
		};
		
		diesel::update(accounts::table)
			.filter(accounts::id.eq(account_id))
			.set(accounts::amount.eq(accounts::amount + v))
			.get_result(self.db)
			.map_err(Into::into)
	}
}

