use crate::repos::common::*;

#[test]
fn create_transaction() {
	let conn = get_db_connection();
	let suite = Suite::setup(&conn);
	let user = suite.create_user();
	
	let checking = suite.create_account(AccountType::Checking, &user);
	
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

