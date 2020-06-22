use crate::repos::common::*;

#[test]
fn create_transaction() {
	let mut fixture = Fixture::new();
	let suite = Suite::setup();
	let user = fixture.create_user();
	
	let checking = fixture.create_checking_account(&user);
	
	let to_id = checking.id;
	
	let got = suite.transaction_repo.create(NewTransaction {
		from_id: None,
		to_id: Some(to_id),
		transaction_type: TransactionType::Deposit,
		amount: BigDecimal::from(250),
	}).unwrap();
	
	let want = Transaction {
		id: got.id,
		from_id: None,
		to_id: Some(to_id),
		transaction_type: TransactionType::Deposit,
		amount: BigDecimal::from(250),
		timestamp: got.timestamp,
	};
	
	assert_eq!(got, want);
}




















































