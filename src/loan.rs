use std::ops::Add;
use std::str::FromStr;

use bigdecimal::{BigDecimal, Zero};
use chrono::Datelike;
use chrono::format::Numeric::Month;
use diesel::{
	deserialize::{self, FromSql},
	PgConnection,
	prelude::*,
	serialize,
	serialize::{Output, ToSql},
	sql_types::Varchar,
};
use diesel::pg::Pg;
use strum;
use strum_macros::{Display, EnumString};

use crate::schema::{loan_payments, loans};
use crate::types::{Date, Id, PgPool, Result};

#[derive(Queryable, Identifiable, Debug)]
pub struct Loan {
	pub id: Id,
	pub user_id: Id,
	pub vault_name: String,
	pub orig_principal: BigDecimal,
	// curr_principal = orig_principal - principal payments + capitalized interest
	pub balance: BigDecimal,
	pub interest_rate: i16,
	pub issue_date: Date,
	pub maturity_date: Date,
	pub payment_frequency: i16,
	pub compound_frequency: i16,
	pub accrued_interest: BigDecimal,
	pub capitalized_interest: BigDecimal,
	pub state: LoanState,
}

#[derive(Insertable)]
#[table_name = "loans"]
pub struct NewLoan {
	pub user_id: uuid::Uuid,
	pub vault_name: String,
	pub orig_principal: BigDecimal,
	pub balance: BigDecimal,
	pub interest_rate: i16,
	pub issue_date: Date,
	pub maturity_date: Date,
	pub payment_frequency: i16,
	pub compound_frequency: i16,
	pub state: LoanState,
}

#[derive(Debug, AsExpression, FromSqlRow, Eq, PartialEq, EnumString, Display)]
#[sql_type = "Varchar"]
#[strum(serialize_all = "snake_case")]
pub enum LoanState {
	PendingApproval,
	Active,
	Paid,
	Default,
}

impl Default for LoanState {
	fn default() -> Self { LoanState::PendingApproval }
}

impl ToSql<Varchar, Pg> for LoanState {
	fn to_sql<W: std::io::Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
		ToSql::<Varchar, Pg>::to_sql(&self.to_string(), out)
	}
}

impl FromSql<Varchar, Pg> for LoanState {
	fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
		let bytes = bytes.ok_or_else(|| "error deserializing from varchar")?;
		let s = std::str::from_utf8(bytes)?;
		
		Ok(LoanState::from_str(s).unwrap())
	}
}


#[derive(Queryable, Identifiable, Debug)]
pub struct LoanPayment {
	pub id: uuid::Uuid,
	pub loan_id: uuid::Uuid,
	pub principal_due: BigDecimal,
	pub interest_due: BigDecimal,
	pub due_date: Date,
	pub principle_transaction_id: Option<uuid::Uuid>,
	pub interest_transaction_id: Option<uuid::Uuid>,
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
	
	pub fn months_til_maturity(&self, previous_payment: Option<&LoanPayment>) -> u16 {
		let start_date: Date;
		if let Some(prev) = previous_payment {
			start_date = prev.due_date;
		} else {
			start_date = self.issue_date;
		}
		let years = self.maturity_date.year() - start_date.year();
		let months = (self.maturity_date.month() - start_date.month()) + (years * 12) as u32;
		months as u16
	}
}


pub struct Repo {
	db: PgPool,
}

impl Repo {
	pub fn new(db: PgPool) -> Self {
		Repo { db }
	}
	
	pub fn create(&self, new_loan: NewLoan) -> Result<Loan> {
		//todo: validate orig_principal == curr_principal
		let conn = &self.db.get()?;
		diesel::insert_into(loans::table)
			.values(&new_loan)
			.get_result(conn)
			.map_err(Into::into)
	}
	
	pub fn find_by_id(&self, id: &uuid::Uuid) -> Result<Loan> {
		let conn = &self.db.get()?;
		loans::table
			.find(id)
			.select(loans::all_columns)
			.first(conn)
			.map_err(Into::into)
	}
	
	pub fn set_state(&self, id: &uuid::Uuid, state: LoanState) -> Result<Loan> {
		let conn = &self.db.get()?;
		diesel::update(loans::table)
			.filter(loans::id.eq(id))
			.set((loans::state.eq(state)))
			.get_result(conn)
			.map_err(Into::into)
	}
	
	pub fn set_accrued_interest(&self, id: &uuid::Uuid, accrued_interest: &BigDecimal) -> Result<Loan> {
		let conn = &self.db.get()?;
		diesel::update(loans::table)
			.filter(loans::id.eq(id))
			.set((loans::accrued_interest.eq(accrued_interest)))
			.get_result(conn)
			.map_err(Into::into)
	}
	
	pub fn decrement(&self, id: &Id, amount: &BigDecimal) -> Result<Loan> {
		let conn = &self.db.get()?;
		diesel::update(loans::table)
			.filter(loans::id.eq(id))
			.set((
				loans::balance.eq(loans::balance + loans::accrued_interest - amount),
				loans::accrued_interest.eq(BigDecimal::zero()),
			))
			.get_result(conn)
			.map_err(Into::into)
	}
}

#[derive(Insertable)]
#[table_name = "loan_payments"]
pub struct NewPayment {
	pub loan_id: uuid::Uuid,
	pub principal_due: BigDecimal,
	pub interest_due: BigDecimal,
	pub due_date: Date,
}

pub struct PaymentRepo {
	db: PgPool,
}

impl PaymentRepo {
	pub fn new(db: PgPool) -> Self {
		PaymentRepo { db }
	}
	
	pub fn create_payment(&self, new_payment: NewPayment) -> Result<LoanPayment> {
		let conn = &self.db.get()?;
		diesel::insert_into(loan_payments::table)
			.values(&new_payment)
			.get_result(conn)
			.map_err(Into::into)
	}
	
	pub fn find_payment_by_id(&self, id: &Id) -> Result<LoanPayment> {
		let conn = &self.db.get()?;
		loan_payments::table
			.find(id)
			.select(loan_payments::all_columns)
			.first(conn)
			.map_err(Into::into)
	}
	
	pub fn find_first_unpaid(&self, loan_id: &Id) -> Result<LoanPayment> {
		let conn = &self.db.get()?;
		loan_payments::table
			.filter((
				loan_payments::loan_id.eq(loan_id)
					.and(loan_payments::principle_transaction_id.is_null())
					.and(loan_payments::interest_transaction_id.is_null())
			))
			.select(loan_payments::all_columns)
			.first(conn)
			.map_err(Into::into)
	}
	
	pub fn find_last_paid(&self, loan_id: &Id) -> Result<LoanPayment> {
		let conn = &self.db.get()?;
		loan_payments::table
			.filter((
				loan_payments::loan_id.eq(loan_id)
					.and(loan_payments::principle_transaction_id.is_not_null())
					.and(loan_payments::interest_transaction_id.is_not_null())
			))
			.select(loan_payments::all_columns)
			.order(loan_payments::due_date.desc())
			.first(conn)
			.map_err(Into::into)
	}
	
	pub fn update_transaction_ids(&self, id: &Id, principle_transaction_id: &Id, interest_transaction_id: &Id) -> Result<LoanPayment> {
		let conn = &self.db.get()?;
		diesel::update(loan_payments::table)
			.filter(loan_payments::id.eq(id))
			.set((
				(loan_payments::principle_transaction_id.eq(principle_transaction_id)),
				(loan_payments::interest_transaction_id.eq(interest_transaction_id)),
			))
			.get_result(conn)
			.map_err(Into::into)
	}
}



