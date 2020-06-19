use std::borrow::Borrow;
use std::fmt::Debug;
use std::ops::Add;
use std::sync::Once;

use bigdecimal::{BigDecimal, Zero};
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, Table};
use diesel::sql_types::Text;

use bank_api::*;
use bank_api::schema::*;

struct Suite<'a> {
	user_repo: UserRepo<'a>,
	account_repo: AccountRepo<'a>,
	conn: &'a PgConnection,
	// users: Vec<User>,
}

impl<'a> Suite<'a> {
	pub fn setup(conn: &'a PgConnection) -> Self {
		Suite::teardown(conn);
		// println!("--- seeding ---");
		// let users = Suite::create_users(conn);
		// users.iter().map(|u| Suite::create_accounts(conn, u.id));
		
		
		let mut suite = Suite {
			user_repo: UserRepo::new(conn),
			account_repo: AccountRepo::new(conn),
			conn,
		};
		
		suite
	}
	
	fn teardown(conn: &PgConnection) {
		let tables = vec!["accounts", "users"];
		println!("--- clean up ---");
		for table in tables {
			diesel::sql_query(format!("DELETE FROM {}", table))
				.execute(conn)
				.map(|n| println!("deleting {} from '{}' table", n, table))
				.expect("deleting db table");
		}
	}
	
	fn create_users(&self) -> Vec<User> {
		let input = vec![
			NewUser {
				email: "vince@gmail.com",
				first_name: "Vincent",
				family_name: "Xiao",
				phone_number: None,
			},
			NewUser {
				email: "jack@gmail.com",
				first_name: "Jack",
				family_name: "Smith",
				phone_number: None,
			},
		];
		
		diesel::insert_into(users::table)
			.values(&input)
			.get_results(self.conn)
			.map(|users: Vec<User>| {
				println!("created {} 'users'", users.len());
				users
			})
			.expect("inserting users")
	}
	
	
	fn create_accounts(&self, user_ids: &Vec<uuid::Uuid>) -> Vec<Account> {
		let mut input: Vec<NewAccount> = Vec::new();
		for &id in user_ids {
			input.append(vec![
				NewAccount {
					user_id: id,
					account_type: AccountType::Checking,
					amount: BigDecimal::from(1000),
				},
				NewAccount {
					user_id: id,
					account_type: AccountType::Savings,
					amount: BigDecimal::from(1000),
				}
			].as_mut()
			);
		}
		
		diesel::insert_into(accounts::table)
			.values(&input)
			.get_results(self.conn)
			.unwrap()
	}
}

#[test]
fn test_suite_setup() {
	let conn = get_db_connection();
	let _suite = Suite::setup(&conn);
}

#[test]
fn insert_user() {
	let conn = get_db_connection();
	let suite = Suite::setup(&conn);
	let user = suite.user_repo.create_user(NewUser {
		email: "vince@gmail.com",
		first_name: "vince",
		family_name: "xiao",
		phone_number: Some("555-5555"),
	}).unwrap();
	
	let got_user = users::table.find(user.id).first::<User>(&conn).unwrap();
	assert_eq!(got_user, user)
}

#[test]
fn find_user_with_key() {
	let conn = get_db_connection();
	let suite = Suite::setup(&conn);
	let users = suite.create_users();
	
	let user = users.first().unwrap();
	let email = user.email.borrow();
	let id = user.id;
	
	// test cases using various UserKeys
	let test_cases = vec![
		UserKey::Email(email),
		UserKey::ID(&id)
	];
	
	
	for user_key in test_cases {
		let got = suite.user_repo.find_user(user_key)
			.expect("found user");
		
		assert_eq!(*user, got)
	}
}

#[test]
fn create_account() {
	let conn = get_db_connection();
	let suite = Suite::setup(&conn);
	let users = suite.create_users();
	
	let user = users.first().unwrap();
	
	let new_account = NewAccount {
		user_id: user.id,
		account_type: AccountType::Checking,
		amount: bigdecimal::BigDecimal::from(100.0),
	};
	
	let want = suite.account_repo.create_account(new_account).unwrap();
	
	let got = accounts::table.find(want.id).first::<Account>(&conn).unwrap();
	assert_eq!(want, got)
}

#[test]
fn find_accounts_for_user() {
	let conn = get_db_connection();
	let suite = Suite::setup(&conn);
	let user_ids: Vec<_> = suite.create_users().iter().map(|u| u.id).collect();
	let want_user_id = user_ids.first().unwrap();
	
	
	let want: Vec<_> = suite.create_accounts(&user_ids).into_iter().filter(|account| account
		.user_id == *want_user_id).collect();
	
	
	let got = suite.account_repo.find_accounts(&want_user_id).unwrap();
	assert_eq!(want, got)
}

#[test]
fn deposit_into_account() {
	let conn = get_db_connection();
	let suite = Suite::setup(&conn);
	let user_ids: Vec<_> = suite.create_users().iter().map(|u| u.id).collect();
	
	let user_id = *user_ids.get(0).unwrap();
	let accounts = suite.create_accounts(&vec![user_id]);
	let account = accounts.first().unwrap();
	
	let deposit_amount = 500;
	let got = suite.account_repo.deposit(&account.id, BigDecimal::from(deposit_amount))
		.unwrap();
	
	let want_amount = (&account.amount) + BigDecimal::from(deposit_amount);
	assert_eq!(got.amount, want_amount)
}



