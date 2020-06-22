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
			transaction_repo: &self.repo_suite.bank_transaction_repo,
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
	s.repo_suite.account_repo.transact(BankTransactionType::Deposit, &account.id, &deposit_amount);
	s.repo_suite.vault_repo.transact(BankTransactionType::Deposit, &s.vault.name, &deposit_amount);
	
	let withdraw_amount = BigDecimal::from(300);
	let account = s.bank_service().withdraw(&account.id, &s.vault.name, &withdraw_amount).unwrap();
	
	assert_eq!(account.amount, deposit_amount - withdraw_amount);
	
	let vault = s.repo_suite.vault_repo.find_by_name(&s.vault.name).unwrap();
	assert_eq!(account.amount, vault.amount);
}

#[test]
fn withdraw_invalid_funds_err() {
	let mut fixture = Fixture::new();
	let s = Suite::setup(&fixture);
	let bob = fixture.user_factory.bob();
	let bob_account = fixture.account_factory.checking_account(bob.id);
	let vault = &s.vault;
	
	let withdraw_amount = BigDecimal::from(500);
	let got_err = s.bank_service().withdraw(&bob_account.id, &vault.name, &withdraw_amount).unwrap_err();
	
	assert_eq!(got_err, Error::new(Kind::InadequateFunds))
}
