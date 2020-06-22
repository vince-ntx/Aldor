use bigdecimal::BigDecimal;
use diesel::PgConnection;

use bank_api::*;

use crate::repos::common::{Fixture, Suite as RepoSuite, TestUsers};

struct Suite {
	pub repo_suite: RepoSuite,
	pub pool: PgPool,
}

impl Suite {
	pub fn new(db: PgPool) -> Self {
		Suite {
			repo_suite: RepoSuite::setup(),
			
			pool: db,
		}
	}
	
	pub fn bank_service(&self) -> BankService {
		BankService::new(NewBankService {
			db: self.pool.clone(),
			user_repo: &self.repo_suite.user_repo,
			account_repo: &self.repo_suite.account_repo,
			transaction_repo: &self.repo_suite.transaction_repo,
		})
	}
}

#[test]
fn deposit() {
	let mut fixture = Fixture::new();
	let s = Suite::new(fixture.pool());
	let user = fixture.create_user();
	
	let account_id = fixture.create_checking_account(&user).id;
	
	let deposit_amount = BigDecimal::from(300);
	let got = s.bank_service().deposit(&account_id, &deposit_amount).unwrap();
	
	assert_eq!(got.amount, deposit_amount);
}

#[test]
fn withdraw() {
	let mut fixture = Fixture::new();
	let s = Suite::new(fixture.pool());
	let (user, account) = fixture.create_user_and_checking_account();
	
	let deposit_amount = BigDecimal::from(500);
	s.repo_suite.account_repo.transact(TransactionType::Deposit, &account.id, &deposit_amount);
	
	let withdraw_amount = BigDecimal::from(300);
	let account = s.bank_service().withdraw(&account.id, &withdraw_amount).unwrap();
	
	assert_eq!(account.amount, deposit_amount - withdraw_amount);
}

#[test]
fn withdraw_invalid_funds_err() {
	let mut fixture = Fixture::new();
	let s = Suite::new(fixture.pool());
	let (user, account) = fixture.create_user_and_checking_account();
	let withdraw_amount = BigDecimal::from(500);
	let got_err = s.bank_service().withdraw(&account.id, &withdraw_amount).unwrap_err();
	
	assert_eq!(got_err, Error::new(Kind::InadequateFunds))
}


