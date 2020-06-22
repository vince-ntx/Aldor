use bank_api::account_transaction::NewAccountTransaction;

use crate::common::*;

#[test]
fn create_account_transaction() {
	let mut fixture = Fixture::new();
	let suite = Suite::setup();
	
	let sender_account = fixture.account_factory.checking_account(
		fixture.user_factory.bob().id
	);
	let receiver_account = fixture.account_factory.checking_account(
		fixture.user_factory.lucy().id
	);
	
	let amount = BigDecimal::from(100);
	
	let got = suite.account_transaction_repo.transfer(NewAccountTransaction {
		sender_id: &sender_account.id,
		receiver_id: &receiver_account.id,
		amount: &amount,
	}).unwrap();
	
	let want = AccountTransaction {
		id: got.id,
		sender_id: sender_account.id,
		receiver_id: receiver_account.id,
		amount,
		created_at: got.created_at,
	};
	
	assert_eq!(got, want);
}
