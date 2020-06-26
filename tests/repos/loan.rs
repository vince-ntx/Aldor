use bank_api::loan::{NewLoan, NewPayment};

use crate::common::*;

#[test]
fn create_loan() {
	let f = Fixture::new();
	let suite = Suite::setup();
	let bob = f.user_factory.bob();
	let vault = f.insert_main_vault(0);
	
	let loan = suite.loan_repo.create(NewLoan {
		user_id: bob.id,
		vault_name: vault.name,
		orig_principal: Default::default(),
		balance: Default::default(),
		interest_rate: 0,
		issue_date: chrono::NaiveDate::from_yo(2020, 1),
		maturity_date: chrono::NaiveDate::from_yo(2020, 1),
		payment_frequency: 0,
		compound_frequency: 0,
		state: Default::default(),
	}).unwrap();
	
	// create loan payment
	let loan_payment = suite.loan_payment_repo.create_payment(NewPayment {
		loan_id: loan.id,
		principal_due: Default::default(),
		interest_due: Default::default(),
		due_date: chrono::NaiveDate::from_yo(2020, 1),
	});
}

