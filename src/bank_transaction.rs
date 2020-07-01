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

use crate::{db, PgPool};
use crate::schema::bank_transactions;
use crate::types::{Result, Time};

#[derive(Queryable, Identifiable, PartialEq, Debug)]
pub struct BankTransaction {
	pub id: uuid::Uuid,
	pub account_id: uuid::Uuid,
	pub vault_name: String,
	pub transaction_type: BankTransactionType,
	pub amount: BigDecimal,
	pub created_at: Time,
}

#[derive(AsExpression, FromSqlRow, Eq, PartialEq, EnumString, Display, Debug)]
#[sql_type = "Varchar"]
#[strum(serialize_all = "snake_case")]
pub enum BankTransactionType {
	Deposit,
	Withdraw,
	LoanPrincipal,
	PrincipalRepayment,
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

pub struct Repo {
	db: PgPool
}

impl Repo {
	pub fn new(db: PgPool) -> Self {
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

#[derive(Insertable)]
#[table_name = "bank_transactions"]
pub struct NewBankTransaction<'a> {
	pub account_id: &'a uuid::Uuid,
	pub vault_name: &'a str,
	pub transaction_type: BankTransactionType,
	pub amount: &'a BigDecimal,
}


