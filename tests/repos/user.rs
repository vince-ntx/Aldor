use std::borrow::Borrow;

use crate::common::*;

#[test]
fn insert_user() {
	let fixture = Fixture::new();
	let suite = Suite::setup();
	let user = suite.user_repo.create_user(NewUser {
		email: "example@gmail.com",
		first_name: "Tom",
		family_name: "Riddle",
		phone_number: Some("555-5555"),
	}).unwrap();
	
	let got_user = users::table.find(user.id).first::<User>(&fixture.conn()).unwrap();
	assert_eq!(got_user, user)
}

#[test]
fn find_user_with_key() {
	let mut fixture = Fixture::new();
	let user = fixture.user_factory.bob();
	
	let suite = Suite::setup();
	
	let email = user.email.borrow();
	let id = user.id;
	
	// test cases using various UserKeys
	let test_cases = vec![
		UserKey::Email(email),
		UserKey::ID(id)
	];
	
	
	for user_key in test_cases {
		let got = suite.user_repo.find_user(user_key)
			.expect("found user");
		
		assert_eq!(user, got)
	}
}

