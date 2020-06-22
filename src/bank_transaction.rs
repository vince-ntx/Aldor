use std::time::SystemTime;

use bigdecimal::BigDecimal;
use diesel::{deserialize, PgConnection, serialize};
use diesel::deserialize::FromSql;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::Varchar;

use crate::{BankTransaction, BankTransactionType, PgPool, Result};
use crate::schema::bank_transactions;

pub struct Repo {
	db: PgPool
}

impl Repo {
	pub fn new(db: PgPool) -> Self {
		Repo { db }
	}
	
	pub fn create(&self, new_transaction: NewBankTransaction) -> Result<BankTransaction> {
		let conn = &self.db.get()?;
		diesel::insert_into(bank_transactions::table)
			.values(&new_transaction)
			.get_result::<BankTransaction>(conn)
			.map_err(Into::into)
	}
}

#[derive(Insertable)]
#[table_name = "bank_transactions"]
pub struct NewBankTransaction<'a> {
	pub account_id: &'a uuid::Uuid,
	pub vault_name: &'a str,
	pub transaction_type: BankTransactionType,
	pub amount: &'a BigDecimal,
}


