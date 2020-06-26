use bigdecimal::{BigDecimal, Zero};
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

use crate::{Date, Id, Loan, LoanPayment, LoanState, PgPool, Result};
use crate::schema::{loan_payments, loans};

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
	// pub state: LoanState,
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
	
	pub fn activate(&self, id: &uuid::Uuid) -> Result<Loan> {
		let conn = &self.db.get()?;
		diesel::update(loans::table)
			.filter(loans::id.eq(id))
			.set((loans::state.eq(LoanState::Active)))
			.get_result(conn)
			.map_err(Into::into)
	}
	
	pub fn update_from_payment(&self, id: &Id, amount: &BigDecimal) -> Result<Loan> {
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



