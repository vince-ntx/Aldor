use std::str::FromStr;
use std::string::ToString;

use bigdecimal::BigDecimal;
use diesel::{
	deserialize,
	pg::Pg,
	prelude::*,
	serialize,
	sql_types::Varchar,
};
use strum;
use strum_macros::{Display, EnumString};

use crate::db;
use crate::schema::bank_transactions;
use crate::types::Time;

/// Transaction between a user's account and the bank
#[derive(Queryable, Identifiable, PartialEq, Debug)]
pub struct BankTransaction {
	pub id: uuid::Uuid,
	/// The user's account id
	pub account_id: uuid::Uuid,
	/// The bank vault's unique identifier
	pub vault_name: String,
	pub transaction_type: BankTransactionType,
	pub amount: BigDecimal,
	pub created_at: Time,
}

#[derive(AsExpression, FromSqlRow, Eq, PartialEq, EnumString, Display, Debug)]
#[sql_type = "Varchar"]
#[strum(serialize_all = "snake_case")]
pub enum BankTransactionType {
	/// A user putting funds into their account
	Deposit,
	/// A user removing funds from their account
	Withdraw,
	/// Funds that are borrowed from the bank
	LoanPrincipal,
	/// Principal repayment on a loan
	PrincipalRepayment,
	/// Interest repayment on a loan
	InterestRepayment,
}


impl serialize::ToSql<Varchar, Pg> for BankTransactionType {
	fn to_sql<W: std::io::Write>(&self, out: &mut serialize::Output<W, Pg>) -> serialize::Result {
		serialize::ToSql::<Varchar, Pg>::to_sql(&self.to_string(), out)
	}
}

impl deserialize::FromSql<Varchar, Pg> for BankTransactionType {
	fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
		let bytes = bytes.ok_or_else(|| "error deserializing from varchar")?;
		let s = std::str::from_utf8(bytes)?;
		
		Ok(BankTransactionType::from_str(s).unwrap())
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

/// Data store implementation for operating on bank_transactions in the database
pub struct Repo {
	db: db::PgPool
}

impl Repo {
	pub fn new(db: db::PgPool) -> Self {
		Repo { db }
	}
	
	pub fn create(&self, new_transaction: NewBankTransaction) -> db::Result<BankTransaction> {
		let conn = &self.db.get()?;
		diesel::insert_into(bank_transactions::table)
			.values(&new_transaction)
			.get_result::<BankTransaction>(conn)
			.map_err(Into::into)
	}
}

#[cfg(test)]
mod tests {
	use crate::testutil::*;
	
	use super::*;
	
	#[test]
	fn create_bank_transaction() {
		let fixture = Fixture::new();
		let suite = Suite::setup();
		let user = fixture.user_factory.bob();
		
		let checking = fixture.account_factory.checking_account(user.id);
		let vault = fixture.insert_main_vault(0);
		
		let amount = BigDecimal::from(250);
		
		let got = suite.bank_transaction_repo.create(NewBankTransaction {
			account_id: &checking.id,
			vault_name: &vault.name,
			transaction_type: BankTransactionType::Deposit,
			amount: &amount,
		}).unwrap();
		
		let want = BankTransaction {
			id: got.id,
			account_id: checking.id,
			vault_name: vault.name,
			transaction_type: BankTransactionType::Deposit,
			amount,
			created_at: got.created_at,
		};
		
		assert_eq!(got, want);
	}
}


