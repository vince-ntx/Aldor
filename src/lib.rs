#![allow(warnings)]
#[macro_use]
extern crate diesel;

use std::borrow::Borrow;
use std::env;
use std::io::Write;
use std::ops::{Add, Div, Mul, Neg, Sub};
use std::str::FromStr;
use std::time::SystemTime;

use bigdecimal::BigDecimal;
use chrono::{Datelike, DateTime, Duration, NaiveDate, TimeZone, Utc};
use diesel::{deserialize::*, deserialize, Queryable, QueryableByName, r2d2, serialize};
pub use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::{Text, Varchar};
use serde::{Deserialize, Serialize};
use strum;
use strum::*;
use strum_macros::{Display, EnumString};
use uuid;
use uuid::Uuid;

use dotenv::dotenv;
use schema::*;

pub use crate::account::*;
use crate::account_transaction::NewAccountTransaction;
pub use crate::bank_transaction::*;
pub use crate::error::*;
use crate::loan::{NewLoan, NewPayment};
pub use crate::schema::*;
pub use crate::user::*;

mod schema;
pub mod error;
pub mod account;
pub mod user;
pub mod bank_transaction;
pub mod account_transaction;
pub mod vault;
pub mod loan;

type Result<T> = std::result::Result<T, error::Error>;
pub type PgPool = r2d2::Pool<ConnectionManager<PgConnection>>;

/// Connect to PostgreSQL database
pub fn get_db_connection() -> PgPool {
	dotenv().ok();
	
	let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
	
	let manager = ConnectionManager::<PgConnection>::new(&database_url);
	let pool = r2d2::Pool::builder().build(manager)
		.expect("Failed to create pool.");
	
	pool
}

pub struct NewBankService<'a> {
	pub db: PgPool,
	pub user_repo: &'a user::Repo,
	pub vault_repo: &'a vault::Repo,
	pub account_repo: &'a account::Repo,
	pub bank_transaction_repo: &'a bank_transaction::Repo,
	pub account_transaction_repo: &'a account_transaction::Repo,
	pub loan_repo: &'a loan::Repo,
	pub loan_payment_repo: &'a loan::PaymentRepo,
}

pub struct BankService<'a> {
	//todo: abstract this out into a trait
	db: PgPool,
	user_repo: &'a user::Repo,
	account_repo: &'a account::Repo,
	vault_repo: &'a vault::Repo,
	bank_transaction_repo: &'a bank_transaction::Repo,
	account_transaction_repo: &'a account_transaction::Repo,
	loan_repo: &'a loan::Repo,
	loan_payments_repo: &'a loan::PaymentRepo,
}

impl<'a> BankService<'a> {
	pub fn new(n: NewBankService<'a>) -> Self {
		BankService {
			db: n.db,
			user_repo: n.user_repo,
			account_repo: n.account_repo,
			vault_repo: n.vault_repo,
			bank_transaction_repo: n.bank_transaction_repo,
			account_transaction_repo: n.account_transaction_repo,
			loan_repo: n.loan_repo,
			loan_payments_repo: n.loan_payment_repo,
		}
	}
	
	pub fn deposit(&self, account_id: &uuid::Uuid, vault_name: &str, amount: &BigDecimal) -> Result<Account> { //todo:: add result
		let conn = &self.db.get()?;
		conn.transaction::<Account, error::Error, _>(|| {
			self.bank_transaction_repo.create(bank_transaction::NewBankTransaction {
				account_id,
				vault_name,
				transaction_type: BankTransactionType::Deposit,
				amount,
			})?;
			
			let account = self.account_repo.increment(account_id, amount)?;
			self.vault_repo.transact(BankTransactionType::Deposit, vault_name, amount)?;
			
			Ok(account)
		})
	}
	
	pub fn withdraw(&self, account_id: &uuid::Uuid, vault_name: &str, amount: &BigDecimal) -> Result<Account> {
		let mut account = self.account_repo.find_by_id(account_id)?;
		if account.amount.lt(amount) {
			return Err(Error::new(Kind::InadequateFunds));
		}
		
		let conn = &self.db.get()?;
		conn.transaction::<(), error::Error, _>(|| {
			self.bank_transaction_repo.create(bank_transaction::NewBankTransaction {
				account_id,
				vault_name,
				transaction_type: BankTransactionType::Withdraw,
				amount,
			})?;
			
			account = self.account_repo.decrement(account_id, amount)?;
			self.vault_repo.transact(BankTransactionType::Withdraw, vault_name, amount)?;
			
			Ok(())
		});
		
		Ok(account)
	}
	
	pub fn send_funds(&self, sender_id: &uuid::Uuid, receiver_id: &uuid::Uuid, amount: &BigDecimal) -> Result<AccountTransaction> {
		let mut sender_account = self.account_repo.find_by_id(sender_id)?;
		if sender_account.amount.lt(amount) {
			return Err(Error::new(Kind::InadequateFunds));
		}
		
		let conn = &self.db.get()?;
		conn.transaction::<AccountTransaction, error::Error, _>(|| {
			let transaction = self.account_transaction_repo.create(NewAccountTransaction {
				sender_id,
				receiver_id,
				amount,
			})?;
			
			self.account_repo.increment(receiver_id, amount)?;
			self.account_repo.decrement(sender_id, amount)?;
			
			Ok(transaction)
		})
	}
	
	
	// create loan
	// create next payment due
	/*
	- principal_due: remaining principle / remaining months
	- interest_due: (remaining principle * intrest rate) / payment_frequency
	- due_date: 1 month + issue_date
	 */
	pub fn approve_loan(&self, new_loan: loan::NewLoan) -> Result<Loan> {
		//todo: create transaction
		let loan = self.loan_repo.create(new_loan)?;
		let principal = &loan.principal;
		
		let principal_due = principal.div(BigDecimal::from(loan.months_til_maturity()));
		let interest_due = principal.mul(loan.interest_rate()) / loan.payment_frequency;
		
		let next_loan_payment = self.loan_payments_repo.create(
			{
				NewPayment {
					loan_id: loan.id,
					principal_due,
					interest_due,
					due_date: Loan::increment_date(&loan.issue_date, 1),
				}
			});
		
		// send funds from vault
		
		
		Ok(loan)
	}
	
	/*
	start db transaction
	create transaction
	subtract from account
	add to bank
	 */
	// pub fn pay_loan_payment_due(&self, loan_payment_id: &uuid::Uuid, account_id: &uuid::Uuid) -> Result<LoanPayment> {
	// 	let loan_payment = self.loan_payments_repo.find_by_id(loan_payment_id)?;
	// 	let loan = self.loan_repo.find_by_id(&loan_payment.loan_id)?;
	// 	let account = self.account_repo.find_by_id(account_id)?;
	//
	// 	let conn = &self.db.get()?;
	// 	conn.transaction::<LoanPayment, error::Error, _>(|| {
	// 		self.bank_transaction_repo.create(NewBankTransaction {
	// 			account_id,
	// 			vault_name: "",
	// 			transaction_type: BankTransactionType::Deposit,
	// 			amount: &Default::default(),
	// 		})
	// 	}
	// }
}

#[derive(Queryable, PartialEq, Debug)]
pub struct Vault {
	pub name: String,
	pub amount: BigDecimal,
}

#[derive(Queryable, Identifiable, PartialEq, Debug)]
pub struct User {
	pub id: uuid::Uuid,
	pub email: String,
	pub first_name: String,
	pub family_name: String,
	pub phone_number: Option<String>,
	/* TODO: add additional info here including
	- date of birth
	- home address
	 */
}


#[derive(Queryable, Identifiable, Associations, PartialEq, Debug)]
#[belongs_to(User)]
pub struct Account {
	pub id: uuid::Uuid,
	pub user_id: uuid::Uuid,
	pub account_type: AccountType,
	pub amount: BigDecimal,
	pub created_at: chrono::DateTime<chrono::Utc>,
	pub is_open: bool,
}

#[derive(AsExpression, FromSqlRow, PartialEq, Debug)]
#[sql_type = "Varchar"]
pub enum AccountType {
	Checking,
	Savings,
}

impl AccountType {
	pub fn as_str(&self) -> &str {
		match self {
			AccountType::Checking => "checking",
			AccountType::Savings => "savings",
		}
	}
}

impl ToSql<Varchar, Pg> for AccountType {
	fn to_sql<W: std::io::Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
		ToSql::<Varchar, Pg>::to_sql(self.as_str(), out)
	}
}

impl FromSql<Varchar, Pg> for AccountType {
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

#[derive(Queryable, Identifiable, PartialEq, Debug)]
pub struct BankTransaction {
	pub id: uuid::Uuid,
	pub account_id: uuid::Uuid,
	pub vault_name: String,
	pub transaction_type: BankTransactionType,
	pub amount: BigDecimal,
	pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, AsExpression, FromSqlRow, Eq, PartialEq, EnumString, Display)]
#[sql_type = "Varchar"]
#[strum(serialize_all = "snake_case")]
pub enum BankTransactionType {
	Deposit,
	Withdraw,
	LoanPrincipal,
	PrincipalRepayment,
	InterestRepayment,
}

impl ToSql<Varchar, Pg> for BankTransactionType {
	fn to_sql<W: std::io::Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
		ToSql::<Varchar, Pg>::to_sql(&self.to_string(), out)
	}
}

impl FromSql<Varchar, Pg> for BankTransactionType {
	fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
		let o = bytes.ok_or_else(|| "error deserializing from varchar")?;
		let s = std::str::from_utf8(o)?;
		
		Ok(BankTransactionType::from_str(s).unwrap())
	}
}

#[derive(Queryable, Identifiable, PartialEq, Debug)]
pub struct AccountTransaction {
	pub id: uuid::Uuid,
	pub sender_id: uuid::Uuid,
	pub receiver_id: uuid::Uuid,
	pub amount: BigDecimal,
	pub created_at: chrono::DateTime<chrono::Utc>,
}

pub type Date = chrono::NaiveDate;

#[derive(Queryable, Identifiable, Debug)]
pub struct Loan {
	pub id: uuid::Uuid,
	pub user_id: uuid::Uuid,
	pub principal: BigDecimal,
	pub interest_rate: i16,
	pub issue_date: Date,
	pub maturity_date: Date,
	pub payment_frequency: i16,
	pub compound_frequency: i16,
	pub accrued_interest: BigDecimal,
	pub active: bool,
}


#[derive(Queryable, Identifiable, Debug)]
pub struct LoanPayment {
	pub id: uuid::Uuid,
	pub loan_id: uuid::Uuid,
	pub principal_due: BigDecimal,
	pub interest_due: BigDecimal,
	pub due_date: Date,
	pub is_paid: bool,
}


impl Loan {
	pub fn increment_date(issue_date: &Date, num_months: u16) -> Date {
		let mut add_years: u32 = (num_months / 12) as u32;
		let mut add_months: u32 = (num_months % 12) as u32;
		
		let maturity_month: u32;
		
		let total_months = issue_date.month().add(add_months as u32);
		if total_months > 12 {
			maturity_month = (total_months / 12);
			add_years += 1;
		} else {
			maturity_month = total_months;
		}
		
		let maturity_year: i32 = issue_date.year() + add_years as i32;
		
		chrono::NaiveDate::from_ymd(maturity_year, maturity_month, issue_date.day())
	}
	
	// Converts interest rate (in basis points) to BigDecimal
	pub fn interest_rate(&self) -> BigDecimal {
		BigDecimal::from(self.interest_rate) / 10000
	}
	
	fn months_til_maturity(&self) -> u16 {
		let years = self.maturity_date.year() - self.issue_date.year();
		let months = (self.maturity_date.month() - self.issue_date.month()) + (years * 12) as u32;
		months as u16
	}
}

