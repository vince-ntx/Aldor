use crate::common::*;

#[test]
fn create_bank_transaction() {
	let mut fixture = Fixture::new();
	let suite = Suite::setup();
	let user = fixture.user_factory.bob();
	
	let checking = fixture.account_factory.checking_account(user.id);
	let vault = fixture.insert_main_vault(0);
	
	let amount = BigDecimal::from(250);
	
	let got = suite.bank_transaction_repo.create(NewBankTransaction {
		account_id: &checking.id,
		vault_name: &vault.name,
		transaction_type: BankTransactionType::Deposit,
		amount: &amount,
	}).unwrap();
	
	let want = BankTransaction {
		id: got.id,
		account_id: checking.id,
		vault_name: vault.name,
		transaction_type: BankTransactionType::Deposit,
		amount,
		created_at: got.created_at,
	};
	
	assert_eq!(got, want);
}




















































