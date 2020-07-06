use std::borrow::Borrow;
use std::ops::Neg;
use std::time::SystemTime;

use bigdecimal::BigDecimal;
use diesel::{
	associations,
	deserialize,
	pg::Pg,
	PgConnection,
	prelude::*,
	serialize,
	sql_types::Varchar,
};
use strum;
use strum_macros::{Display, EnumString};

use crate::db;
use crate::schema::accounts;
use crate::types::Time;

/// The user's financial account maintained by the bank to hold and manage funds
/// A user may have multiple accounts
#[derive(Queryable, Identifiable, PartialEq, Debug)]
pub struct Account {
	pub id: uuid::Uuid,
	/// the account owner's user id
	pub user_id: uuid::Uuid,
	pub account_type: AccountType,
	/// the account balance
	pub amount: BigDecimal,
	pub created_at: Time,
	/// indicates whether an account is currently open/closed for use
	pub is_open: bool,
}

#[derive(Insertable)]
#[table_name = "accounts"]
pub struct NewAccount {
	pub user_id: uuid::Uuid,
	pub account_type: AccountType,
}

#[derive(AsExpression, FromSqlRow, PartialEq, EnumString, Display, Debug)]
#[sql_type = "Varchar"]
#[strum(serialize_all = "snake_case")]
pub enum AccountType {
	Checking,
	Savings,
}

impl serialize::ToSql<Varchar, Pg> for AccountType {
	fn to_sql<W: std::io::Write>(&self, out: &mut serialize::Output<W, Pg>) -> serialize::Result {
		serialize::ToSql::<Varchar, Pg>::to_sql(&self.to_string(), out)
	}
}

impl deserialize::FromSql<Varchar, Pg> for AccountType {
	fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
		let o = bytes.ok_or_else(|| "error deserializing from varchar")?;
		let x = std::str::from_utf8(o)?;
		match x {
			"checking" => Ok(AccountType::Checking),
			"savings" => Ok(AccountType::Savings),
			_ => Err("invalid account type".into())
		}
	}
}

/// Data store implementation for operating on accounts in the database
pub struct Repo {
	db: db::PgPool,
}

impl Repo {
	pub fn new(db: db::PgPool) -> Self {
		Repo { db }
	}
	
	pub fn create_account(&self, new_account: NewAccount) -> db::Result<Account> {
		let conn = &self.db.get()?;
		diesel::insert_into(accounts::table)
			.values(&new_account)
			.get_result(conn)
			.map_err(Into::into)
	}
	
	pub fn find_accounts(&self, user_id: &uuid::Uuid) -> db::Result<Vec<Account>> {
		let conn = &self.db.get()?;
		accounts::table
			.filter(accounts::user_id.eq(user_id))
			.select((accounts::all_columns))
			.load::<Account>(conn)
			.map_err(Into::into)
	}
	
	pub fn find_by_id(&self, account_id: &uuid::Uuid) -> db::Result<Account> {
		let conn = &self.db.get()?;
		accounts::table
			.filter(accounts::id.eq(account_id))
			.select((accounts::all_columns))
			.first::<Account>(conn)
			.map_err(Into::into)
	}
	
	pub fn increment(&self, account_id: &uuid::Uuid, amount: &BigDecimal) -> db::Result<Account> {
		self.transact(account_id, amount)
	}
	
	pub fn decrement(&self, account_id: &uuid::Uuid, amount: &BigDecimal) -> db::Result<Account> {
		let neg = amount.neg();
		self.transact(account_id, &neg)
	}
	
	/// Helper method for incrementing/decrementing funds from an account
	fn transact(&self, account_id: &uuid::Uuid, amount: &BigDecimal) -> db::Result<Account> {
		let conn = &self.db.get()?;
		diesel::update(accounts::table)
			.filter(accounts::id.eq(account_id))
			.set(accounts::amount.eq(accounts::amount + amount))
			.get_result(conn)
			.map_err(Into::into)
	}
}

#[cfg(test)]
mod tests {
	use crate::testutil::*;
	
	use super::*;
	
	#[test]
	fn create_account() {
		let fixture = Fixture::new();
		let suite = Suite::setup();
		let user = fixture.user_factory.bob();
		
		let new_account = NewAccount {
			user_id: user.id,
			account_type: AccountType::Checking,
		};
		
		let want = suite.account_repo.create_account(new_account).unwrap();
		
		let got = accounts::table.find(want.id).first::<Account>(&fixture.conn()).unwrap();
		assert_eq!(want, got)
	}
	
	#[test]
	fn find_accounts_for_user() {
		let fixture = Fixture::new();
		let suite = Suite::setup();
		let user = fixture.user_factory.bob();
		
		let mut want = Vec::new();
		let checking = fixture.account_factory.checking_account(user.id);
		let savings = fixture.account_factory.checking_account(user.id);
		want.push(checking);
		want.push(savings);
		
		let got = suite.account_repo.find_accounts(&user.id).unwrap();
		
		assert_eq!(want, got)
	}
	
	#[test]
	fn account_deposit_and_withdrawal() {
		let fixture = Fixture::new();
		let suite = Suite::setup();
		let user = fixture.user_factory.bob();
		
		let checking = fixture.account_factory.checking_account(user.id);
		
		// deposit
		let deposit_amount = BigDecimal::from(500);
		let got = suite.account_repo.increment(&checking.id, &deposit_amount).unwrap();
		
		let want_amount = (checking.amount) + BigDecimal::from(deposit_amount);
		assert_eq!(got.amount, want_amount, "account's amount should be equal to the deposit");
		
		let withdraw_amount = BigDecimal::from(250);
		let got = suite.account_repo.increment(&checking.id, &withdraw_amount).unwrap();
		
		let want_amount = (&want_amount) - withdraw_amount;
		assert_eq!(got.amount, want_amount, "account's amount should be equal to (deposit - withdrawal)");
	}
}

