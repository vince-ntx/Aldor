use std::ops::Neg;

use bigdecimal::BigDecimal;
use diesel::prelude::*;

use crate::{BankTransactionType, PgPool, Result, Vault};
use crate::schema::vaults;

#[derive(Insertable)]
#[table_name = "vaults"]
pub struct NewVault<'a> {
	pub name: &'a str,
	#[column_name = "amount"]
	pub initial_amount: BigDecimal,
}

pub struct Repo {
	db: PgPool,
}

impl Repo {
	pub fn new(db: PgPool) -> Self { Repo { db } }
	
	pub fn find_by_name(&self, name: &str) -> Result<Vault> {
		let conn = &self.db.get()?;
		vaults::table
			.filter(vaults::name.eq(name))
			.select((vaults::all_columns))
			.first::<Vault>(conn)
			.map_err(Into::into)
	}
	
	pub fn transact(&self, k: BankTransactionType, vault_name: &str, value: &BigDecimal) -> Result<Vault> {
		let conn = &self.db.get()?;
		let neg_value = value.neg();
		let v = match k {
			BankTransactionType::Deposit => value,
			BankTransactionType::Withdraw => &neg_value,
		};
		
		diesel::update(vaults::table)
			.filter(vaults::name.eq(vault_name))
			.set(vaults::amount.eq(vaults::amount + v))
			.get_result(conn)
			.map_err(Into::into)
	}
}