use std::ops::Neg;
use std::time::SystemTime;

use bigdecimal::BigDecimal;
use diesel::{
	deserialize,
	deserialize::FromSql,
	pg::Pg,
	PgConnection,
	prelude::*,
	serialize,
	serialize::{Output, ToSql},
	sql_types::Varchar,
};

use crate::{Account, AccountType, Result, schema::*, TransactionType};

#[derive(Insertable)]
#[table_name = "accounts"]
pub struct NewAccount {
	pub user_id: uuid::Uuid,
	pub account_type: AccountType,
}

pub struct Repo<'a> {
	db: &'a PgConnection,
}

impl<'a> Repo<'a> {
	pub fn new(db: &'a PgConnection) -> Self {
		Repo { db }
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
	
	pub fn transact(&self, k: TransactionType, account_id: &uuid::Uuid, value: &BigDecimal) ->
	Result<Account> {
		let neg_value = value.neg();
		let v = match k {
			TransactionType::Deposit => value,
			TransactionType::Withdraw => &neg_value,
		};
		
		diesel::update(accounts::table)
			.filter(accounts::id.eq(account_id))
			.set(accounts::amount.eq(accounts::amount + v))
			.get_result(self.db)
			.map_err(Into::into)
	}
}

