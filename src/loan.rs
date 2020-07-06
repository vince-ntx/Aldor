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

/// Loan issued by the bank to a user
/// Loans are amortized and the borrower must make periodic payments that cover both principal and interest
#[derive(Queryable, Identifiable, Debug)]
pub struct Loan {
	pub id: Id,
	/// id of the user (borrower)
	pub user_id: Id,
	/// unique name of the vault where loan funds will be drawn from
	pub vault_name: String,
	/// the amount of the loan that would be repaid over the lifetime of the loan
	pub orig_principal: BigDecimal,
	/// the balance is equal to (original principal + accrued/capitalized interest) - (principal + interest payments)
	pub balance: BigDecimal,
	/// the interest rate is represented in basis points (one hundreth of one percent)
	/// e.g. 2% is 200 basis points, .5% is 50 basis points
	interest_rate: i16,
	/// the date in which the loan is issued and begins accruing interest
	pub issue_date: Date,
	/// the date in which the final payment is due
	pub maturity_date: Date,
	/// the payment frequency represents the number of months between payments
	pub payment_frequency: i16,
	/// the compound frequency represents the number of months between compounding
	///
	/// when a loan compounds, the unpaid accrued interest on the loan is capitalized and added to loan balance
	/// and future interest accrued will take into account the capitalized interest
	pub compound_frequency: i16,
	/// the accrued interest on the loan
	pub accrued_interest: BigDecimal,
	/// tracks the amount of capitalized interest on the loan
	pub capitalized_interest: BigDecimal,
	/// the state of the loan
	pub state: LoanState,
}

impl Loan {
	/// Gets the interest rate and converts it from basis points to BigDecimal
	pub fn interest_rate(&self) -> BigDecimal {
		BigDecimal::from(self.interest_rate) / 10_000
	}
	
	/// Calculates the months til maturity from the current date
	pub fn months_til_maturity(&self, curr_date: Date) -> u16 {
		let years = self.maturity_date.year() - curr_date.year();
		let months = (self.maturity_date.month() - curr_date.month()) + (years * 12) as u32;
		months as u16
	}
	
	/// Calculates the principle due for a pay period
	///
	/// # Arguments
	/// `curr_date` - determines the months left til maturity and is used to calculate the principal payment
	pub fn principal_due(&self, curr_date: Date) -> BigDecimal {
		let months_til_maturity = self.months_til_maturity(curr_date);
		(&self.balance)
			.div(&BigDecimal::from(months_til_maturity))
			.mul(BigDecimal::from(self.payment_frequency))
	}
}


#[derive(Debug, AsExpression, FromSqlRow, Eq, PartialEq, EnumString, Display)]
#[sql_type = "Varchar"]
#[strum(serialize_all = "snake_case")]
pub enum LoanState {
	/// The loan is pending approval
	PendingApproval,
	/// Active indicates the loan balance is being repaid within the specified terms
	Active,
	/// All principal and interest payments have been fulfilled
	Paid,
	/// The borrower has failed to make an principal or interest payment within the specified terms
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

/// Data store implementation for operating on loans in the database
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


/// Loan payment due based on the terms of the loan
#[derive(Queryable, Identifiable, Debug)]
pub struct LoanPayment {
	pub id: uuid::Uuid,
	pub loan_id: uuid::Uuid,
	pub principal_due: BigDecimal,
	pub interest_due: BigDecimal,
	pub due_date: Date,
	/// id of the principal payment transaction
	pub principle_transaction_id: Option<uuid::Uuid>,
	/// id of the interest payment transaction
	pub interest_transaction_id: Option<uuid::Uuid>,
}


#[derive(Insertable)]
#[table_name = "loan_payments"]
pub struct NewPayment {
	pub loan_id: uuid::Uuid,
	pub principal_due: BigDecimal,
	pub interest_due: BigDecimal,
	pub due_date: Date,
}

/// Data store implementation for operating on loan_payments in the database
pub struct PaymentRepo {
	db: db::PgPool,
}

impl PaymentRepo {
	pub fn new(db: db::PgPool) -> Self {
		PaymentRepo { db }
	}
	
	pub fn create(&self, new_payment: NewPayment) -> db::Result<LoanPayment> {
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
	
	/// Finds the first unpaid loan payment due
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
	
	/// Finds the most recently paid loan payment
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
	
	/// Updates the principal and interest due on the loan payment
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
		let loan_payment = suite.loan_payment_repo.create(NewPayment {
			loan_id: loan.id,
			principal_due: Default::default(),
			interest_due: Default::default(),
			due_date: chrono::NaiveDate::from_yo(2020, 1),
		});
	}
}
