use std::ops::Neg;

use bigdecimal::BigDecimal;
use diesel::prelude::*;

use crate::{BankTransactionType, PgPool, Result, Vault};
use crate::BankTransactionType::PrincipalRepayment;
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
	
	pub fn increment(&self, vault_name: &str, amount: &BigDecimal) -> Result<Vault> {
		self.transact(vault_name, amount)
	}
	
	pub fn decrement(&self, vault_name: &str, amount: &BigDecimal) -> Result<Vault> {
		let neg = amount.neg();
		self.transact(vault_name, &neg)
	}
	
	fn transact(&self, vault_name: &str, amount: &BigDecimal) -> Result<Vault> {
		let conn = &self.db.get()?;
		diesel::update(vaults::table)
			.filter(vaults::name.eq(vault_name))
			.set(vaults::amount.eq(vaults::amount + amount))
			.get_result(conn)
			.map_err(Into::into)
	}
}
