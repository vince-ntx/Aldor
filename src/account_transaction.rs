use std::time::SystemTime;

use bigdecimal::BigDecimal;
use diesel::{deserialize, PgConnection, serialize};
use diesel::deserialize::FromSql;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::Varchar;

use crate::{Date, Id, PgPool, Result};
use crate::schema::account_transactions;

#[derive(Queryable, Identifiable, PartialEq, Debug)]
pub struct AccountTransaction {
	pub id: Id,
	pub sender_id: Id,
	pub receiver_id: Id,
	pub amount: BigDecimal,
	pub created_at: Date,
}

pub struct Repo {
	db: PgPool
}

impl Repo {
	pub fn new(db: PgPool) -> Self {
		Repo { db }
	}
	
	pub fn create(&self, new_transaction: NewAccountTransaction) -> Result<AccountTransaction> {
		let conn = &self.db.get()?;
		diesel::insert_into(account_transactions::table)
			.values(&new_transaction)
			.get_result::<>(conn)
			.map_err(Into::into)
	}
}

#[derive(Insertable)]
#[table_name = "account_transactions"]
pub struct NewAccountTransaction<'a> {
	pub sender_id: &'a uuid::Uuid,
	pub receiver_id: &'a uuid::Uuid,
	pub amount: &'a BigDecimal,
}

