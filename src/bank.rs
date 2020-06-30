use std::env;
use std::ops::{Add, Div, Mul, Neg, Sub};

use bigdecimal::{BigDecimal, Signed, Zero};
use diesel::Connection;

use crate::{account_transaction, loan};
use crate::account::{self, Account};
use crate::account_transaction::{AccountTransaction, NewAccountTransaction};
use crate::bank_transaction::{self, BankTransactionType, NewBankTransaction};
use crate::error::{Error, Kind};
use crate::loan::{Loan, LoanPayment, LoanState, NewPayment};
use crate::types::{Date, Id, PgPool, Result};
use crate::user::{self, User};
use crate::vault::{self, Vault};

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
	
	pub fn withdraw(&self, account_id: &uuid::Uuid, vault_name: &str, amount: &BigDecimal) -> Result<Account> {
		let mut account = self.account_repo.find_by_id(account_id)?;
		if account.amount.lt(amount) {
			return Err(Error::new(Kind::InadequateFunds));
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
	
	pub fn send_funds(&self, sender_id: &uuid::Uuid, receiver_id: &uuid::Uuid, amount: &BigDecimal) -> Result<AccountTransaction> {
		let mut sender_account = self.account_repo.find_by_id(sender_id)?;
		if sender_account.amount.lt(amount) {
			return Err(Error::new(Kind::InadequateFunds));
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
	
	/*
	- principal_due: remaining principle / remaining months
	- interest_due: (remaining principle * interest rate) / payment_frequency
	- due_date: 1 month + issue_date
	 */
	pub fn disburse_loan(&self, loan: &Loan, account_id: &Id) -> Result<()> {
		//todo: validate that the account belongs to the user under loan.user_id
		let conn = &self.db.get()?;
		
		conn.transaction::<_, Error, _>(|| {
			self.vault_repo.decrement(&loan.vault_name, &loan.orig_principal)?;
			self.account_repo.increment(account_id, &loan.orig_principal)?;
			
			// self.create_next_loan_payment(&loan)?;
			Ok(())
		})
	}
	
	pub fn create_next_loan_payment(&self, loan: &Loan) -> Result<LoanPayment> {
		// pull up previous
		let previous_payment = self.loan_payments_repo.find_last_paid(&loan.id).ok();
		
		
		let balance = &loan.balance;
		//todo: assumes most recent as been paid
		let mut months_til_maturity = loan.months_til_maturity(previous_payment.as_ref());
		
		let principal_due = balance.div(&BigDecimal::from(months_til_maturity));
		//todo: interest needs to account for periods less than a year
		// let interest_due = balance.mul(loan.interest_rate()) / loan.payment_frequency;
		let interest_due = loan.accrued_interest.clone();
		let mut due_date: Date;
		
		let num_months = loan.payment_frequency / 12;
		
		if let Some(prev) = previous_payment {
			due_date = Loan::increment_date(&prev.due_date, num_months as u16);
		} else {
			due_date = Loan::increment_date(&loan.issue_date, num_months as u16);
		}
		
		self.loan_payments_repo.create_payment(
			{
				NewPayment {
					loan_id: loan.id,
					principal_due,
					interest_due,
					due_date,
				}
			})
	}
	
	pub fn accrue(&self, loan: &Loan) -> Result<Loan> {
		let accrued_interest = (&loan.balance).mul(loan.interest_rate()).div(BigDecimal::from(12));
		let loan = self.loan_repo.set_accrued_interest(&loan.id, &accrued_interest)?;
		// let loan_payment = self.loan_payments_repo.find_first_unpaid(&loan.id)?;
		Ok(loan)
	}
	
	pub fn pay_loan_payment_due(&self, loan_payment_id: &uuid::Uuid, account_id: &uuid::Uuid) -> Result<LoanPayment> {
		//todo: validate we're within loan payment's due date range
		let mut loan_payment = self.loan_payments_repo.find_payment_by_id(loan_payment_id)?;
		let mut loan = self.loan_repo.find_by_id(&loan_payment.loan_id)?;
		let account = self.account_repo.find_by_id(account_id)?;
		
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
			
			// deduct funds from the account
			self.account_repo.decrement(account_id, &total_payment)?;
			
			// increment funds in bank vault
			self.vault_repo.increment(&loan.vault_name, &total_payment)?;
			
			loan = self.loan_repo.decrement(&loan.id, &total_payment)?;
			
			// update loan payment
			loan_payment = self.loan_payments_repo.update_transaction_ids(loan_payment_id,
																		  &principal_transaciton.id,
																		  &interest_transaction.id)?;
			
			// todo: update loan, mark as paid in full if this is the last payment due
			if loan.balance.is_zero() {
				loan = self.loan_repo.set_state(&loan.id, LoanState::Paid)?;
			}
			
			Ok(loan_payment)
		})
	}
}

