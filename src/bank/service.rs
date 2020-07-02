use std::env;
use std::ops::{Add, Div, Mul, Neg, Sub};

use bigdecimal::{BigDecimal, Signed, Zero};
use diesel::Connection;

use crate::{account_transaction, db, loan};
use crate::account::{self, Account};
use crate::account_transaction::{AccountTransaction, NewAccountTransaction};
use crate::bank_transaction::{self, BankTransactionType, NewBankTransaction};
use crate::loan::{Loan, LoanPayment, LoanState, NewPayment};
use crate::types::{Date, DateExt, Id};
use crate::user::{self, User};
use crate::vault::{self, Vault};

use super::error::{Error, ErrorKind};

pub type Result<T> = std::result::Result<T, Error>;

/// Service for performing banking operations
pub struct Service<'a> {
	db: db::PgPool,
	user_repo: &'a user::Repo,
	account_repo: &'a account::Repo,
	vault_repo: &'a vault::Repo,
	bank_transaction_repo: &'a bank_transaction::Repo,
	account_transaction_repo: &'a account_transaction::Repo,
	loan_repo: &'a loan::Repo,
	loan_payments_repo: &'a loan::PaymentRepo,
	calendar: &'a dyn Calendar,
}

/// Parameter object for creating a new Service
pub struct NewService<'a> {
	pub db: db::PgPool,
	pub user_repo: &'a user::Repo,
	pub vault_repo: &'a vault::Repo,
	pub account_repo: &'a account::Repo,
	pub bank_transaction_repo: &'a bank_transaction::Repo,
	pub account_transaction_repo: &'a account_transaction::Repo,
	pub loan_repo: &'a loan::Repo,
	pub loan_payment_repo: &'a loan::PaymentRepo,
	pub calendar: &'a dyn Calendar,
}

impl<'a> Service<'a> {
	pub fn new(v: NewService<'a>) -> Self {
		Service {
			db: v.db,
			user_repo: v.user_repo,
			account_repo: v.account_repo,
			vault_repo: v.vault_repo,
			bank_transaction_repo: v.bank_transaction_repo,
			account_transaction_repo: v.account_transaction_repo,
			loan_repo: v.loan_repo,
			loan_payments_repo: v.loan_payment_repo,
			calendar: v.calendar,
		}
	}
	
	/// Deposit funds to a user's account
	///
	/// # Arguments
    /// * `account_id` - user's account id in which funds belong to
    /// * `vault_name` - vault's unique name where the funds are held for safekeeping
    /// * `amount` - amount deposited
	pub fn deposit(&self, account_id: &uuid::Uuid, vault_name: &str, amount: &BigDecimal) -> Result<Account> {
		let conn = &self.db.get()?;
		conn.transaction::<Account, Error, _>(|| {
			self.bank_transaction_repo.create(bank_transaction::NewBankTransaction {
				account_id,
				vault_name,
				transaction_type: BankTransactionType::Deposit,
				amount,
			})?;
			
			let account = self.account_repo.increment(account_id, amount)?;
			self.vault_repo.increment(vault_name, amount)?;
			
			Ok(account)
		})
	}
	
	/// Withdraw funds from a user's account
	///
	/// # Arguments
    /// * `account_id` - user's account id that the funds belong to
    /// * `vault_name` - vault's unique name where the funds are stored and withdrawn from
    /// * `amount` - amount withdrawn
	pub fn withdraw(&self, account_id: &uuid::Uuid, vault_name: &str, amount: &BigDecimal) -> Result<Account> {
		let mut account = self.account_repo.find_by_id(account_id)?;
		if account.amount.lt(amount) {
			return Err(Error::new(ErrorKind::InadequateFunds));
		}
		
		let conn = &self.db.get()?;
		conn.transaction::<(), Error, _>(|| {
			self.bank_transaction_repo.create(bank_transaction::NewBankTransaction {
				account_id,
				vault_name,
				transaction_type: BankTransactionType::Withdraw,
				amount,
			})?;
			
			account = self.account_repo.decrement(account_id, amount)?;
			self.vault_repo.decrement(vault_name, amount)?;
			
			Ok(())
		});
		
		Ok(account)
	}
	
	/// Transfer funds from account to account
	/// This allows users to transfer funds to one another
	///
	/// # Arguments
    /// * `account_id` - user's account id in which funds belong to
    /// * `vault_name` - vault's unique name where the funds are transferred to for safekeeping and use by the bank
    /// * `amount` - amount deposited
	pub fn send_funds(&self, sender_id: &uuid::Uuid, receiver_id: &uuid::Uuid, amount: &BigDecimal) -> Result<AccountTransaction> {
		let mut sender_account = self.account_repo.find_by_id(sender_id)?;
		if sender_account.amount.lt(amount) {
			return Err(Error::new(ErrorKind::InadequateFunds));
		}
		
		let conn = &self.db.get()?;
		conn.transaction::<AccountTransaction, Error, _>(|| {
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
	
	/// Transfer the loan principal from the bank to the borrower's account
	///
	/// # Arguments
    /// * `loan` - the loan with information about the bank, user, and loan principal
    /// * `account_id` - the user's account id that funds will be transferred to
	pub fn disburse_loan(&self, loan: &Loan, account_id: &Id) -> Result<()> {
		//todo:
		// - validate the loan has been granted approval
		// - validate that the account belongs to the user under loan.user_id
		let conn = &self.db.get()?;
		
		conn.transaction::<_, Error, _>(|| {
			self.vault_repo.decrement(&loan.vault_name, &loan.orig_principal)?;
			self.account_repo.increment(account_id, &loan.orig_principal)?;
			
			Ok(())
		})
	}
	
	/// Gets the next loan payment due for the loan
	///
	/// Creates the loan payment if it doesn't exist
	/// Updates the loan payment based on the loan's current balance and accrued interest
	pub fn get_next_loan_payment(&self, loan: &Loan) -> Result<LoanPayment> {
		let loan_payment = match self.loan_payments_repo.find_by_id(&loan.id) {
			Ok(val) => val,
			Err(e) => return match e {
				db::Error::RecordNotFound => self.create_next_loan_payment(loan),
				_ => Err(Error::from(e))
			},
		};
		let loan_payment = self.update_loan_payment(loan, &loan_payment.id)?;
		Ok(loan_payment)
	}
	
	/// Gets the next loan payment due for the loan
	///
	/// Creates the loan payment if it doesn't exist
	/// Updates the loan payment based on the loan's current balance and accrued interest
	pub fn update_loan_payment(&self, loan: &Loan, loan_payment_id: &Id) -> Result<LoanPayment> {
		self.loan_payments_repo.set_dues(loan_payment_id,
										 &loan.principal_due(self.calendar.current_date()),
										 &loan.accrued_interest).map_err(Into::into)
	}
	
	/// Calculate and accrue interest on the loan
	/// Updates the loan with the current accrued interest
	pub fn accrue(&self, loan: &Loan) -> Result<Loan> {
		let divisor = BigDecimal::from(12 / loan.payment_frequency);
		let accrued_interest = (&loan.balance).mul(loan.interest_rate()).div(divisor);
		self.loan_repo.set_accrued_interest(&loan.id, &accrued_interest).map_err(Into::into)
	}
	
	/// Pay the current loan payment dues
	///
	/// # Arguments
	/// `loan_payment_id` - id of loan payment
	/// `account_id` - id of the user's account that will be used to pay the dues
	pub fn pay_loan_payment_due(&self, loan_payment_id: &uuid::Uuid, account_id: &uuid::Uuid) -> Result<LoanPayment> {
		//todo: validate we're within loan payment's due date range
		let mut loan_payment = self.loan_payments_repo.find_by_id(loan_payment_id)?;
		let mut loan = self.loan_repo.find_by_id(&loan_payment.loan_id)?;
		
		let conn = &self.db.get()?;
		conn.transaction::<LoanPayment, Error, _>(|| {
			let principal_transaciton = self.bank_transaction_repo.create(NewBankTransaction {
				account_id,
				vault_name: &loan.vault_name,
				transaction_type: BankTransactionType::PrincipalRepayment,
				amount: &loan_payment.principal_due,
			})?;
			let interest_transaction = self.bank_transaction_repo.create(NewBankTransaction {
				account_id,
				vault_name: &loan.vault_name,
				transaction_type: BankTransactionType::InterestRepayment,
				amount: &loan_payment.interest_due,
			})?;
			
			let total_payment = &loan_payment.principal_due + &loan_payment.interest_due;
			
			// deduct funds from the user's account
			self.account_repo.decrement(account_id, &total_payment)?;
			
			// increment funds in the bank's vault
			self.vault_repo.increment(&loan.vault_name, &total_payment)?;
			
			// decrement the dues from the loan
			loan = self.loan_repo.decrement(&loan.id, &total_payment)?;
			
			// attach the transaction ids to the loan payment
			loan_payment = self.loan_payments_repo.set_transaction_ids(loan_payment_id,
																	   &principal_transaciton.id,
																	   &interest_transaction.id)?;
			
			if loan.balance.is_zero() {
				loan = self.loan_repo.set_state(&loan.id, LoanState::Paid)?;
			}
			
			// invalid balance check
			assert!(!loan.balance.is_negative(), "invalid state: loan balance should never be negative");
			
			Ok(loan_payment)
		})
	}
	
	/// Create the next loan payment due on the loan
	fn create_next_loan_payment(&self, loan: &Loan) -> Result<LoanPayment> {
		// Look up the previous payment to see if we are creating the first payment due on this loan
		let previous_payment = match self.loan_payments_repo.find_last_paid(&loan.id) {
			Ok(v) => Some(v),
			Err(db::Error::RecordNotFound) => None,
			Err(e) => return Err(e.into())
		};
		
		let mut due_date: Date;
		let months_to_add = loan.payment_frequency;
		due_date = match previous_payment {
			Some(prev) => prev.due_date.increment_date_by_months(months_to_add as u16),
			None => loan.issue_date.increment_date_by_months(months_to_add as u16),
		};
		
		if due_date.gt(&loan.maturity_date) {
			let msg = format!("due date({}) exceeds maturity date({})", due_date, loan.maturity_date);
			return Err(Error::new(ErrorKind::InvalidDate(msg)));
		}
		
		let principal_due = loan.principal_due(self.calendar.current_date());
		let interest_due = loan.accrued_interest.clone();
		
		self.loan_payments_repo.create_payment(
			{
				NewPayment {
					loan_id: loan.id,
					principal_due,
					interest_due,
					due_date,
				}
			}).map_err(Into::into)
	}
}

pub trait Calendar {
	/// Gets the current date
	fn current_date(&self) -> Date {
		chrono::Utc::today().naive_utc()
	}
}

