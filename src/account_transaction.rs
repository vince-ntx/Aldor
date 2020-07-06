use std::time::SystemTime;

use bigdecimal::BigDecimal;
use diesel::{deserialize, PgConnection, serialize};
use diesel::deserialize::FromSql;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::Varchar;

use crate::db;
use crate::schema::account_transactions;
use crate::types::{Id, Time};

/// Transaction between accounts
/// The accounts can be:
/// 	- held by two different users
/// 	- held by the same user
#[derive(Queryable, Identifiable, PartialEq, Debug)]
pub struct AccountTransaction {
	pub id: Id,
	/// Sender's account id
	pub sender_id: Id,
	/// Receiver's account id
	pub receiver_id: Id,
	pub amount: BigDecimal,
	pub created_at: Time,
}

#[derive(Insertable)]
#[table_name = "account_transactions"]
pub struct NewAccountTransaction<'a> {
	pub sender_id: &'a uuid::Uuid,
	pub receiver_id: &'a uuid::Uuid,
	pub amount: &'a BigDecimal,
}

pub struct Repo {
	db: db::PgPool
}

/// Data store implementation for operating on account_transactions in the database
impl Repo {
	pub fn new(db: db::PgPool) -> Self {
		Repo { db }
	}
	
	pub fn create(&self, new_transaction: NewAccountTransaction) -> db::Result<AccountTransaction> {
		let conn = &self.db.get()?;
		diesel::insert_into(account_transactions::table)
			.values(&new_transaction)
			.get_result::<>(conn)
			.map_err(Into::into)
	}
}

#[cfg(test)]
mod tests {
	use crate::testutil::*;
	
	use super::*;
	
	#[test]
	fn create_account_transaction() {
		let fixture = Fixture::new();
		let suite = Suite::setup();
		
		let sender_account = fixture.account_factory.checking_account(
			fixture.user_factory.bob().id
		);
		let receiver_account = fixture.account_factory.checking_account(
			fixture.user_factory.lucy().id
		);
		
		let amount = BigDecimal::from(100);
		
		let got = suite.account_transaction_repo.create(NewAccountTransaction {
			sender_id: &sender_account.id,
			receiver_id: &receiver_account.id,
			amount: &amount,
		}).unwrap();
		
		let want = AccountTransaction {
			id: got.id,
			sender_id: sender_account.id,
			receiver_id: receiver_account.id,
			amount,
			created_at: got.created_at,
		};
		
		assert_eq!(got, want);
	}
}
