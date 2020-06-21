use bigdecimal::BigDecimal;
use diesel::PgConnection;

use bank_api::*;

use crate::repos::common::{Suite as RepoSuite, TestUsers};

struct Suite<'a> {
	bank_service: BankService<'a>,
}

impl<'a> Suite<'a> {
	pub fn new(repos: &'a RepoSuite) -> Self {
		Suite {
			bank_service: BankService::new(NewBankService {
				db: &repos.conn,
				user_repo: &repos.user_repo,
				account_repo: &repos.account_repo,
				transaction_repo: &repos.transaction_repo,
			}),
		}
	}
}


#[test]
fn deposit() {
	let conn = get_db_connection();
	let repos = RepoSuite::setup(&conn);
	let suite = Suite::new(&repos);
	let users = repos.create_users();
	
	let user = users.get(TestUsers::email_vince).unwrap();
	let account_id = repos.create_account(AccountType::Checking, user).id;
	
	let deposit_amount = BigDecimal::from(300);
	let got = suite.bank_service.deposit(&account_id, &deposit_amount).unwrap();
	
	assert_eq!(got.amount, deposit_amount);
}

#[test]
fn withdraw() {
	let conn = get_db_connection();
	let repos = RepoSuite::setup(&conn);
	let suite = Suite::new(&repos);
	let (user, account) = repos.create_user_and_account();
	
	let deposit_amount = BigDecimal::from(500);
	repos.account_repo.transact(TransactionType::Deposit, &account.id, &deposit_amount);
	
	let withdraw_amount = BigDecimal::from(300);
	let account = suite.bank_service.withdraw(&account.id, &withdraw_amount).unwrap();
	
	assert_eq!(account.amount, deposit_amount - withdraw_amount);
}

#[test]
fn withdraw_invalid_funds_err() {
	let conn = get_db_connection();
	let repos = RepoSuite::setup(&conn);
	let suite = Suite::new(&repos);
	let (user, account) = repos.create_user_and_account();
}


