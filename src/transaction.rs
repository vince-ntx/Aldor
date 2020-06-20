use std::time::SystemTime;

use bigdecimal::BigDecimal;
use diesel::{deserialize, PgConnection, serialize};
use diesel::deserialize::FromSql;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::Varchar;

use crate::Result;
use crate::schema::transactions;

#[derive(Debug, AsExpression, FromSqlRow, PartialEq)]
#[sql_type = "Varchar"]
pub enum Type {
	Deposit,
	Withdraw,
}

impl Type {
	pub fn as_str(&self) -> &str {
		match self {
			Type::Deposit => "deposit",
			Type::Withdraw => "withdraw",
		}
	}
}


impl ToSql<Varchar, Pg> for Type {
	fn to_sql<W: std::io::Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
		ToSql::<Varchar, Pg>::to_sql(self.as_str(), out)
	}
}

impl FromSql<Varchar, Pg> for Type {
	fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
		let o = bytes.ok_or_else(|| "error deserializing from varchar")?;
		let x = std::str::from_utf8(o)?;
		match x {
			"deposit" => Ok(Type::Deposit),
			"withdraw" => Ok(Type::Withdraw),
			_ => Err("invalid transaction key".into())
		}
	}
}

#[derive(Insertable)]
#[table_name = "transactions"]
pub struct NewTransaction {
	pub from_id: Option<uuid::Uuid>,
	pub to_id: Option<uuid::Uuid>,
	pub transaction_type: Type,
	pub amount: BigDecimal,
}

#[derive(Queryable, Identifiable, PartialEq, Debug)]
pub struct Transaction {
	pub id: uuid::Uuid,
	pub from_id: Option<uuid::Uuid>,
	pub to_id: Option<uuid::Uuid>,
	pub transaction_type: Type,
	pub amount: BigDecimal,
	pub timestamp: SystemTime,
}

pub struct Repo<'a> {
	db: &'a PgConnection,
}

impl<'a> Repo<'a> {
	pub fn new(db: &'a PgConnection) -> Self {
		Repo { db }
	}
	
	pub fn create(&self, new_transaction: NewTransaction) -> Result<Transaction> {
		diesel::insert_into(transactions::table)
			.values(&new_transaction)
			.get_result::<Transaction>(self.db)
			.map_err(Into::into)
	}
}

