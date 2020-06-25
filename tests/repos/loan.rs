use bank_api::loan::{NewLoan, NewLoanPayment};

use crate::common::*;

#[test]
fn create_loan() {
	let f = Fixture::new();
	let suite = Suite::setup();
	let bob = f.user_factory.bob();
	
	let loan = suite.loan_repo.create(NewLoan {
		user_id: bob.id,
		principal: Default::default(),
		interest_rate: 0,
		issue_date: chrono::NaiveDate::from_yo(2020, 1),
		maturity_date: chrono::NaiveDate::from_yo(2020, 1),
		payment_frequency: 0,
		compound_frequency: 0,
	}).unwrap();
	
	// create loan payment
	let loan_payment = suite.loan_payment_repo.create(NewLoanPayment {
		loan_id: loan.id,
		principal_due: Default::default(),
		interest_due: Default::default(),
		due_date: chrono::NaiveDate::from_yo(2020, 1),
	});
}

