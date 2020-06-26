use bigdecimal::BigDecimal;

use bank_api::*;
use bank_api::vault::NewVault;

use crate::common::{Fixture, Suite as RepoSuite, TestUsers};

struct Suite<'a> {
	pub repos: RepoSuite,
	pub fixture: &'a Fixture,
}

impl<'a> Suite<'a> {
	pub fn setup(fixture: &'a Fixture) -> Self {
		let repo_suite = RepoSuite::setup();
		
		Suite {
			repos: repo_suite,
			fixture,
		}
	}
	
	pub fn bank_service(&self) -> BankService {
		BankService::new(NewBankService {
			db: self.fixture.pool.clone(),
			user_repo: &self.repos.user_repo,
			account_repo: &self.repos.account_repo,
			vault_repo: &self.repos.vault_repo,
			bank_transaction_repo: &self.repos.bank_transaction_repo,
			account_transaction_repo: &self.repos.account_transaction_repo,
			loan_repo: &self.repos.loan_repo,
			loan_payment_repo: &self.repos.loan_payment_repo,
		})
	}
}

#[test]
fn deposit() {
	let f = Fixture::new();
	let suite = Suite::setup(&f);
	
	let bob = f.user_factory.bob();
	let bob_account = f.account_factory.checking_account(bob.id);
	let vault = f.insert_main_vault(0);
	
	
	let deposit_amount = BigDecimal::from(300);
	let bob_account = suite.bank_service().deposit(&bob_account.id, &vault.name, &deposit_amount).unwrap();
	assert_eq!(bob_account.amount, deposit_amount);
	
	let vault = suite.repos.vault_repo.find_by_name(&vault.name).unwrap();
	assert_eq!(bob_account.amount, vault.amount);
}

#[test]
fn withdraw() {
	let f = Fixture::new();
	let s = Suite::setup(&f);
	
	let user = f.user_factory.bob();
	let account = f.account_factory.checking_account(user.id);
	let vault = f.insert_main_vault(0);
	
	let deposit_amount = BigDecimal::from(500);
	s.repos.account_repo.increment(&account.id, &deposit_amount);
	s.repos.vault_repo.increment(&vault.name, &deposit_amount);
	
	let withdraw_amount = BigDecimal::from(300);
	let account = s.bank_service().withdraw(&account.id, &vault.name, &withdraw_amount).unwrap();
	
	assert_eq!(account.amount, deposit_amount - withdraw_amount);
	
	let vault = s.repos.vault_repo.find_by_name(&vault.name).unwrap();
	assert_eq!(account.amount, vault.amount);
}

#[test]
fn withdraw_invalid_funds_err() {
	let f = Fixture::new();
	let s = Suite::setup(&f);
	let bob = f.user_factory.bob();
	let bob_account = f.account_factory.checking_account(bob.id);
	let vault = f.insert_main_vault(0);
	
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
	s.repos.account_repo.increment(sender_id, &bob_initial_amount);
	
	let transfer_amount = BigDecimal::from(250);
	let transaction = s.bank_service().send_funds(sender_id, receiver_id, &transfer_amount).unwrap();
	
	let bob_account = s.repos.account_repo.find_by_id(sender_id).unwrap();
	assert_eq!(bob_account.amount, &bob_initial_amount - &transfer_amount);
	
	let lucy_account = s.repos.account_repo.find_by_id(receiver_id).unwrap();
	assert_eq!(lucy_account.amount, transfer_amount);
	
	/* expect error on overdrawn account */
	let transfer_amount = BigDecimal::from(1_000);
	let err = s.bank_service().send_funds(sender_id, receiver_id, &transfer_amount).unwrap_err();
	assert_eq!(err, Error::new(Kind::InadequateFunds))
}


/*
create loan
approve loan
pay loan
check vault and user account's state
 */
#[test]
fn pay_loan_payment_due() -> Result<()> {
	let f = Fixture::new();
	let s = Suite::setup(&f);
	let vault = f.insert_main_vault(0);
	
	let bob = f.user_factory.bob();
	let orig_principal = BigDecimal::from(1000);
	let issue_date = Date::from_ymd(2020, 1, 1);
	let loan = s.repos.loan_repo.create(loan::NewLoan {
		user_id: bob.id,
		vault_name: vault.name,
		orig_principal: orig_principal.clone(),
		balance: orig_principal.clone(),
		interest_rate: 200,
		issue_date,
		maturity_date: Loan::increment_date(&issue_date, 12),
		payment_frequency: 1,
		compound_frequency: 1,
		state: Default::default(),
	})?;
	let loan = s.repos.loan_repo.find_by_id(&loan.id)?;
	assert_eq!(loan.state, LoanState::PendingApproval);
	
	// activate loan
	s.repos.loan_repo.activate(&loan.id)?;
	
	// disburse funds
	let bob_account = f.account_factory.checking_account(bob.id);
	s.bank_service().disburse_loan(&loan, &bob_account.id);
	
	// check that first loan pay
	let first_payment_due = s.repos.loan_payment_repo.find_first(&loan.id)?;
	let first_payment_due = s.bank_service().pay_loan_payment_due(&first_payment_due.id, &bob_account.id)?;
	
	Ok(())
}





























































