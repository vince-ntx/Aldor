use bigdecimal::BigDecimal;
use diesel::PgConnection;

use bank_api::*;

use crate::repos::common::{Fixture, Suite as RepoSuite, TestUsers};

struct Suite<'a> {
	bank_service: BankService<'a>,
}

impl<'a> Suite<'a> {
	pub fn new(repos: &'a RepoSuite) -> Self {
		Suite {
			bank_service: BankService::new(NewBankService {
				db: &repos.fixture.conn,
				user_repo: &repos.user_repo,
				account_repo: &repos.account_repo,
				transaction_repo: &repos.transaction_repo,
			}),
		}
	}
}


#[test]
fn deposit() {
	let mut fixture = Fixture::new();
	let repos = RepoSuite::setup(&fixture);
	let suite = Suite::new(&repos);
	let user = fixture.create_user();
	
	let account_id = repos.create_account(AccountType::Checking, &user).id;
	
	let deposit_amount = BigDecimal::from(300);
	let got = suite.bank_service.deposit(&account_id, &deposit_amount).unwrap();
	
	assert_eq!(got.amount, deposit_amount);
}

#[test]
fn withdraw() {
	let mut fixture = Fixture::new();
	let repos = RepoSuite::setup(&fixture);
	let suite = Suite::new(&repos);
	let (user, account) = fixture.create_user_and_account();
	
	let deposit_amount = BigDecimal::from(500);
	repos.account_repo.transact(TransactionType::Deposit, &account.id, &deposit_amount);
	
	let withdraw_amount = BigDecimal::from(300);
	let account = suite.bank_service.withdraw(&account.id, &withdraw_amount).unwrap();
	
	assert_eq!(account.amount, deposit_amount - withdraw_amount);
}

#[test]
fn withdraw_invalid_funds_err() {
	let mut fixture = Fixture::new();
	let repos = RepoSuite::setup(&fixture);
	let suite = Suite::new(&repos);
	let (user, account) = fixture.create_user_and_account();
}


