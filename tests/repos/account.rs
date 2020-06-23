use crate::common::*;

#[test]
fn create_account() {
	let mut fixture = Fixture::new();
	let suite = Suite::setup();
	let user = fixture.user_factory.bob();
	
	let new_account = NewAccount {
		user_id: user.id,
		account_type: AccountType::Checking,
	};
	
	let want = suite.account_repo.create_account(new_account).unwrap();
	
	let got = accounts::table.find(want.id).first::<Account>(&fixture.conn()).unwrap();
	assert_eq!(want, got)
}

#[test]
fn find_accounts_for_user() {
	let mut fixture = Fixture::new();
	let suite = Suite::setup();
	let user = fixture.user_factory.bob();
	
	let mut want = Vec::new();
	let checking = fixture.account_factory.checking_account(user.id);
	let savings = fixture.account_factory.checking_account(user.id);
	want.push(checking);
	want.push(savings);
	
	let got = suite.account_repo.find_accounts(&user.id).unwrap();
	
	assert_eq!(want, got)
}

#[test]
fn account_deposit_and_withdrawal() {
	let mut fixture = Fixture::new();
	let suite = Suite::setup();
	let user = fixture.user_factory.bob();
	
	let checking = fixture.account_factory.checking_account(user.id);
	
	// deposit
	let deposit_amount = BigDecimal::from(500);
	let got = suite.account_repo.increment(&checking.id, &deposit_amount).unwrap();
	
	let want_amount = (checking.amount) + BigDecimal::from(deposit_amount);
	assert_eq!(got.amount, want_amount, "account's amount should be equal to the deposit");
	
	let withdraw_amount = BigDecimal::from(250);
	let got = suite.account_repo.increment(&checking.id, &withdraw_amount).unwrap();
	
	let want_amount = (&want_amount) - withdraw_amount;
	assert_eq!(got.amount, want_amount, "account's amount should be equal to (deposit - withdrawal)");
}

