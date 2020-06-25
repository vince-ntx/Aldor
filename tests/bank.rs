use bigdecimal::BigDecimal;
use diesel::PgConnection;
use diesel::prelude::*;

use bank_api::*;
use bank_api::vault::NewVault;

use crate::common::{Fixture, Suite as RepoSuite, TestUsers};

struct Suite<'a> {
	pub repo_suite: RepoSuite,
	pub vault: Vault,
	pub fixture: &'a Fixture,
}

impl<'a> Suite<'a> {
	pub fn setup(fixture: &'a Fixture) -> Self {
		let repo_suite = RepoSuite::setup();
		let vault = fixture.insert_main_vault(0);
		
		Suite {
			repo_suite,
			vault,
			fixture,
		}
	}
	
	pub fn bank_service(&self) -> BankService {
		BankService::new(NewBankService {
			db: self.fixture.pool.clone(),
			user_repo: &self.repo_suite.user_repo,
			account_repo: &self.repo_suite.account_repo,
			vault_repo: &self.repo_suite.vault_repo,
			bank_transaction_repo: &self.repo_suite.bank_transaction_repo,
			account_transaction_repo: &self.repo_suite.account_transaction_repo,
			loan_repo: &self.repo_suite.loan_repo,
			loan_payment_repo: &self.repo_suite.loan_payment_repo,
		})
	}
}

#[test]
fn deposit() {
	let f = Fixture::new();
	let suite = Suite::setup(&f);
	
	let bob = f.user_factory.bob();
	let bob_account = f.account_factory.checking_account(bob.id);
	let vault = &suite.vault;
	
	
	let deposit_amount = BigDecimal::from(300);
	let bob_account = suite.bank_service().deposit(&bob_account.id, &vault.name, &deposit_amount).unwrap();
	assert_eq!(bob_account.amount, deposit_amount);
	
	let vault = suite.repo_suite.vault_repo.find_by_name(&vault.name).unwrap();
	assert_eq!(bob_account.amount, vault.amount);
}

#[test]
fn withdraw() {
	let f = Fixture::new();
	let s = Suite::setup(&f);
	
	let user = f.user_factory.bob();
	let account = f.account_factory.checking_account(user.id);
	
	let deposit_amount = BigDecimal::from(500);
	s.repo_suite.account_repo.increment(&account.id, &deposit_amount);
	s.repo_suite.vault_repo.transact(BankTransactionType::Deposit, &s.vault.name, &deposit_amount);
	
	let withdraw_amount = BigDecimal::from(300);
	let account = s.bank_service().withdraw(&account.id, &s.vault.name, &withdraw_amount).unwrap();
	
	assert_eq!(account.amount, deposit_amount - withdraw_amount);
	
	let vault = s.repo_suite.vault_repo.find_by_name(&s.vault.name).unwrap();
	assert_eq!(account.amount, vault.amount);
}

#[test]
fn withdraw_invalid_funds_err() {
	let fixture = Fixture::new();
	let s = Suite::setup(&fixture);
	let bob = fixture.user_factory.bob();
	let bob_account = fixture.account_factory.checking_account(bob.id);
	let vault = &s.vault;
	
	let withdraw_amount = BigDecimal::from(500);
	let got_err = s.bank_service().withdraw(&bob_account.id, &vault.name, &withdraw_amount).unwrap_err();
	
	assert_eq!(got_err, Error::new(Kind::InadequateFunds))
}

#[test]
fn send_funds() {
	let f = Fixture::new();
	let s = Suite::setup(&f);
	
	let bob = f.user_factory.bob();
	let bob_account = f.account_factory.checking_account(bob.id);
	let sender_id = &bob_account.id;
	
	
	let lucy = f.user_factory.lucy();
	let lucy_account = f.account_factory.checking_account(lucy.id);
	let receiver_id = &lucy_account.id;
	
	let bob_initial_amount = BigDecimal::from(500);
	s.repo_suite.account_repo.increment(sender_id, &bob_initial_amount);
	
	let transfer_amount = BigDecimal::from(250);
	let transaction = s.bank_service().send_funds(sender_id, receiver_id, &transfer_amount).unwrap();
	
	let bob_account = s.repo_suite.account_repo.find_account(sender_id).unwrap();
	assert_eq!(bob_account.amount, &bob_initial_amount - &transfer_amount);
	
	let lucy_account = s.repo_suite.account_repo.find_account(receiver_id).unwrap();
	assert_eq!(lucy_account.amount, transfer_amount);
	
	/* expect error on overdrawn account */
	let transfer_amount = BigDecimal::from(1_000);
	let err = s.bank_service().send_funds(sender_id, receiver_id, &transfer_amount).unwrap_err();
	assert_eq!(err, Error::new(Kind::InadequateFunds))
}

#[test]
fn approve_loan() {
	let f = Fixture::new();
	let s = Suite::setup(&f);
	
	let bob = f.user_factory.bob();
	let issue_date = Date::from_ymd(2020, 1, 1);
	
	s.bank_service().approve_loan(loan::NewLoan {
		user_id: bob.id,
		principal: BigDecimal::from(1000),
		interest_rate: 200,
		issue_date,
		maturity_date: Loan::increment_date(&issue_date, 12),
		payment_frequency: 1,
		compound_frequency: 1,
	});
}



















































