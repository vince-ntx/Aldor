use std::time::SystemTime;

use bigdecimal::BigDecimal;
use diesel::{deserialize, PgConnection, serialize};
use diesel::deserialize::FromSql;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::Varchar;

use crate::{PgPool, Result, Transaction, TransactionType};
use crate::schema::transactions;

pub struct Repo {
	db: PgPool
}

impl Repo {
	pub fn new(db: PgPool) -> Self {
		Repo { db }
	}
	
	pub fn create(&self, new_transaction: NewTransaction) -> Result<Transaction> {
		let conn = &self.db.get()?;
		diesel::insert_into(transactions::table)
			.values(&new_transaction)
			.get_result::<Transaction>(conn)
			.map_err(Into::into)
	}
}

#[derive(Insertable)]
#[table_name = "transactions"]
pub struct NewTransaction {
	pub from_id: Option<uuid::Uuid>,
	pub to_id: Option<uuid::Uuid>,
	pub transaction_type: TransactionType,
	pub amount: BigDecimal,
}


