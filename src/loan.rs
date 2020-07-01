use std::ops::{Add, Div, Mul};
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

use crate::db;
use crate::schema::{loan_payments, loans};
use crate::types::{Date, Id};

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

impl Loan {
	// Converts interest rate (in basis points) to BigDecimal
	pub fn interest_rate(&self) -> BigDecimal {
		BigDecimal::from(self.interest_rate) / 10000
	}
	
	pub fn months_til_maturity(&self, curr_date: Date) -> u16 {
		let years = self.maturity_date.year() - curr_date.year();
		let months = (self.maturity_date.month() - curr_date.month()) + (years * 12) as u32;
		months as u16
	}
	
	pub fn principal_due(&self, curr_date: Date) -> BigDecimal {
		let months_til_maturity = self.months_til_maturity(curr_date);
		(&self.balance)
			.div(&BigDecimal::from(months_til_maturity))
			.mul(BigDecimal::from(self.payment_frequency))
	}
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


pub struct Repo {
	db: db::PgPool,
}

impl Repo {
	pub fn new(db: db::PgPool) -> Self {
		Repo { db }
	}
	
	pub fn create(&self, new_loan: NewLoan) -> db::Result<Loan> {
		//todo: validate orig_principal == curr_principal
		let conn = &self.db.get()?;
		diesel::insert_into(loans::table)
			.values(&new_loan)
			.get_result(conn)
			.map_err(Into::into)
	}
	
	pub fn find_by_id(&self, id: &uuid::Uuid) -> db::Result<Loan> {
		let conn = &self.db.get()?;
		loans::table
			.find(id)
			.select(loans::all_columns)
			.first(conn)
			.map_err(Into::into)
	}
	
	pub fn set_state(&self, id: &uuid::Uuid, state: LoanState) -> db::Result<Loan> {
		let conn = &self.db.get()?;
		diesel::update(loans::table)
			.filter(loans::id.eq(id))
			.set((loans::state.eq(state)))
			.get_result(conn)
			.map_err(Into::into)
	}
	
	pub fn set_accrued_interest(&self, id: &uuid::Uuid, accrued_interest: &BigDecimal) -> db::Result<Loan> {
		let conn = &self.db.get()?;
		diesel::update(loans::table)
			.filter(loans::id.eq(id))
			.set((loans::accrued_interest.eq(accrued_interest)))
			.get_result(conn)
			.map_err(Into::into)
	}
	
	pub fn decrement(&self, id: &Id, amount: &BigDecimal) -> db::Result<Loan> {
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
	db: db::PgPool,
}

impl PaymentRepo {
	pub fn new(db: db::PgPool) -> Self {
		PaymentRepo { db }
	}
	
	pub fn create_payment(&self, new_payment: NewPayment) -> db::Result<LoanPayment> {
		let conn = &self.db.get()?;
		diesel::insert_into(loan_payments::table)
			.values(&new_payment)
			.get_result(conn)
			.map_err(Into::into)
	}
	
	pub fn find_by_id(&self, id: &Id) -> db::Result<LoanPayment> {
		let conn = &self.db.get()?;
		loan_payments::table
			.find(id)
			.select(loan_payments::all_columns)
			.first(conn)
			.map_err(Into::into)
	}
	
	pub fn find_first_unpaid(&self, loan_id: &Id) -> db::Result<LoanPayment> {
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
	
	pub fn find_last_paid(&self, loan_id: &Id) -> db::Result<LoanPayment> {
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
	
	pub fn set_transaction_ids(&self, id: &Id, principle_transaction_id: &Id, interest_transaction_id: &Id) -> db::Result<LoanPayment> {
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
	
	pub fn set_dues(&self, id: &Id, principal_due: &BigDecimal, interest_due: &BigDecimal) -> db::Result<LoanPayment> {
		let conn = &self.db.get()?;
		diesel::update(loan_payments::table)
			.filter(loan_payments::id.eq(id))
			.set((
				(loan_payments::principal_due.eq(principal_due)),
				(loan_payments::interest_due.eq(interest_due))
			))
			.get_result(conn)
			.map_err(Into::into)
	}
}


#[cfg(test)]
mod tests {
	use crate::testutil::*;
	
	use super::*;
	
	#[test]
	fn create_loan() {
		let f = Fixture::new();
		let suite = Suite::setup();
		let bob = f.user_factory.bob();
		let vault = f.insert_main_vault(0);
		
		let loan = suite.loan_repo.create(NewLoan {
			user_id: bob.id,
			vault_name: vault.name,
			orig_principal: Default::default(),
			balance: Default::default(),
			interest_rate: 0,
			issue_date: chrono::NaiveDate::from_yo(2020, 1),
			maturity_date: chrono::NaiveDate::from_yo(2020, 1),
			payment_frequency: 0,
			compound_frequency: 0,
			state: Default::default(),
		}).unwrap();
		
		// create loan payment
		let loan_payment = suite.loan_payment_repo.create_payment(NewPayment {
			loan_id: loan.id,
			principal_due: Default::default(),
			interest_due: Default::default(),
			due_date: chrono::NaiveDate::from_yo(2020, 1),
		});
	}
}
