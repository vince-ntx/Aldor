use crate::repos::common::*;

#[test]
fn create_account() {
	let conn = get_db_connection();
	let suite = Suite::setup(&conn);
	let users = suite.create_users();
	
	let user = users.get(TestUsers::email_vince).unwrap();
	
	let new_account = NewAccount {
		user_id: user.id,
		account_type: AccountType::Checking,
	};
	
	let want = suite.account_repo.create_account(new_account).unwrap();
	
	let got = accounts::table.find(want.id).first::<Account>(&conn).unwrap();
	assert_eq!(want, got)
}

#[test]
fn find_accounts_for_user() {
	let conn = get_db_connection();
	let suite = Suite::setup(&conn);
	let user = suite.create_user();
	
	let mut want = Vec::new();
	let checking = suite.create_account(AccountType::Checking, &user);
	let savings = suite.create_account(AccountType::Savings, &user);
	want.push(checking);
	want.push(savings);
	
	let got = suite.account_repo.find_accounts(&user.id).unwrap();
	
	assert_eq!(want, got)
}

#[test]
fn account_deposit_and_withdrawal() {
	let conn = get_db_connection();
	let suite = Suite::setup(&conn);
	let user = suite.create_user();
	
	let checking = suite.create_account(AccountType::Checking, &user);
	
	// deposit
	let deposit_amount = BigDecimal::from(500);
	let got = suite.account_repo.transact(TransactionType::Deposit, &checking.id, &deposit_amount).unwrap();
	
	let want_amount = (checking.amount) + BigDecimal::from(deposit_amount);
	assert_eq!(got.amount, want_amount, "account's amount should be equal to the deposit");
	
	let withdraw_amount = BigDecimal::from(250);
	let got = suite.account_repo.transact(TransactionType::Withdraw, &checking.id, &withdraw_amount).unwrap();
	
	let want_amount = (&want_amount) - withdraw_amount;
	assert_eq!(got.amount, want_amount, "account's amount should be equal to (deposit - withdrawal)");
}

