use bigdecimal::BigDecimal;
use diesel::{
	deserialize,
	deserialize::FromSql,
	pg::Pg,
	PgConnection,
	prelude::*,
	serialize,
	serialize::{Output, ToSql},
	sql_types::Varchar,
};

use crate::{Date, Loan, LoanPayment, PgPool, Result};
use crate::schema::{loan_payments, loans};

#[derive(Insertable)]
#[table_name = "loans"]
pub struct NewLoan {
	pub user_id: uuid::Uuid,
	pub principal: BigDecimal,
	pub interest_rate: i16,
	pub issue_date: Date,
	pub maturity_date: Date,
	pub payment_frequency: i16,
	pub compound_frequency: i16,
}

pub struct Repo {
	db: PgPool,
}

impl Repo {
	pub fn new(db: PgPool) -> Self {
		Repo { db }
	}
	
	pub fn create(&self, new_loan: NewLoan) -> Result<Loan> {
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
	
	pub fn create(&self, new_payment: NewPayment) -> Result<LoanPayment> {
		let conn = &self.db.get()?;
		
		diesel::insert_into(loan_payments::table)
			.values(&new_payment)
			.get_result(conn)
			.map_err(Into::into)
	}
	
	pub fn find_by_id(&self, id: &uuid::Uuid) -> Result<LoanPayment> {
		let conn = &self.db.get()?;
		
		loan_payments::table
			.find(id)
			.select(loan_payments::all_columns)
			.first(conn)
			.map_err(Into::into)
	}
}

