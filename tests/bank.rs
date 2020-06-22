use bigdecimal::BigDecimal;
use diesel::PgConnection;

use bank_api::*;

use crate::repos::common::{Fixture, Suite as RepoSuite, TestUsers};

struct Suite<'a> {
	bank_service: BankService<'a>,
}

impl<'a> Suite<'a> {
	pub fn new(db: PgPool, repo_suite: &'a RepoSuite) -> Self {
		Suite {
			bank_service: BankService::new(NewBankService {
				db,
				user_repo: &repo_suite.user_repo,
				account_repo: &repo_suite.account_repo,
				transaction_repo: &repo_suite.transaction_repo,
			}),
		}
	}
}


#[test]
fn deposit() {
	let mut fixture = Fixture::new();
	let repo_suite = RepoSuite::setup();
	let suite = Suite::new(fixture.pool.clone(), &repo_suite);
	let user = fixture.create_user();
	
	let account_id = fixture.create_checking_account(&user).id;
	
	let deposit_amount = BigDecimal::from(300);
	let got = suite.bank_service.deposit(&account_id, &deposit_amount).unwrap();
	
	assert_eq!(got.amount, deposit_amount);
}

#[test]
fn withdraw() {
	let mut fixture = Fixture::new();
	let repo_suite = RepoSuite::setup();
	let suite = Suite::new(fixture.pool.clone(), &repo_suite);
	let (user, account) = fixture.create_user_and_account();
	
	let deposit_amount = BigDecimal::from(500);
	repo_suite.account_repo.transact(TransactionType::Deposit, &account.id, &deposit_amount);
	
	let withdraw_amount = BigDecimal::from(300);
	let account = suite.bank_service.withdraw(&account.id, &withdraw_amount).unwrap();
	
	assert_eq!(account.amount, deposit_amount - withdraw_amount);
}

#[test]
fn withdraw_invalid_funds_err() {
	// let mut fixture = Fixture::new();
	// let repos = RepoSuite::setup();
	// let suite = Suite::new(&repos);
	// let (user, account) = fixture.create_user_and_account();
}


