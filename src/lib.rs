#![allow(warnings)]
#[macro_use]
extern crate diesel;

use std::borrow::Borrow;
use std::env;
use std::error::Error;
use std::io::Write;
use std::ops::{Add, Neg};
use std::str::FromStr;
use std::time::SystemTime;

use bigdecimal::BigDecimal;
use diesel::{deserialize::*, deserialize, Queryable, QueryableByName, serialize};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::{Text, Varchar};
use serde::{Deserialize, Serialize};
use uuid;
use uuid::Uuid;

use dotenv::dotenv;
use schema::*;

mod schema;
mod errors;
mod account;
mod user;
mod transaction;
mod bank;


/// Connect to PostgreSQL database
pub fn get_db_connection() -> PgConnection {
	dotenv().ok();
	
	let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
	
	PgConnection::establish(&database_url).expect(&format!("error connecting to {}", database_url))
}

type Result<T> = std::result::Result<T, errors::Error>;

pub struct BankingService<'a> {
	db: &'a PgConnection,
	user_repo: &'a user::UserRepo<'a>,
	account_repo: &'a account::AccountRepo<'a>,
	transaction_repo: &'a transaction::Repo<'a>,
}

impl<'a> BankingService<'a> {
	pub fn deposit(&self, account_id: &uuid::Uuid, amount: &BigDecimal) { //todo:: add result
		self.db.transaction::<(), diesel::result::Error, _>(|| {
			self.transaction_repo.create(transaction::NewTransaction {
				from_id: None,
				to_id: Some(account_id.clone()),
				transaction_type: transaction::Type::Deposit,
				amount: amount.to_owned(),
			});
			
			self.account_repo.transact(transaction::Type::Deposit, account_id, amount);
			
			Ok(())
		});
	}
	
	pub fn withdraw(&self, account_id: &uuid::Uuid, amount: &BigDecimal) -> Result<account::Account> {
		let mut account = self.account_repo.find_account(account_id).expect("get account");
		if account.amount.lt(amount) {}
		
		
		self.db.transaction::<(), errors::Error, _>(|| {
			self.transaction_repo.create(transaction::NewTransaction {
				from_id: Some(account_id.clone()),
				to_id: None,
				transaction_type: transaction::Type::Withdraw,
				amount: amount.to_owned(),
			})?;
			
			account = self.account_repo.transact(transaction::Type::Withdraw, account_id, amount)?;
			
			Ok(())
		});
		
		Ok(account)
	}
	
	fn t(&self) {}
}

#[cfg(test)]
mod tests {
	use super::*;
	
	#[test]
	fn t() {
		let conn = get_db_connection();
		let user_repo = user::UserRepo::new(&conn);
		let account_repo = account::AccountRepo::new(&conn);
		let transaction_repo = transaction::Repo::new(&conn);
		let banking_service = BankingService {
			db: &conn,
			user_repo: &user_repo,
			account_repo: &account_repo,
			transaction_repo: &transaction_repo,
		};
		
		let user = user_repo.create_user(user::NewUser {
			email: "gmail",
			first_name: "vince",
			family_name: "xiao",
			phone_number: None,
		}).unwrap();
		let account = account_repo.create_account(account::NewAccount {
			user_id: user.id,
			account_type: account::AccountType::Checking,
		}).unwrap();
		
		banking_service.deposit(&account.id, &BigDecimal::from(300))
	}
}


