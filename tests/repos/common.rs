use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

pub use bigdecimal::BigDecimal;
use diesel::PgConnection;
pub use diesel::prelude::*;

pub use bank_api::*;
use bank_api::AccountType::Checking;

pub struct TestUsers {}

impl<'a> TestUsers {
	pub const email_vince: &'a str = "vince@gmail.com";
	pub const email_jack: &'a str = "jack@gmail.com";
}

pub struct Fixture<'a> {
	pub conn: PgConnection,
	user_gen: Box<dyn FnMut() -> User + 'a>,
}

impl<'a> Fixture<'_> {
	pub fn new() -> Self {
		let conn = get_db_connection();
		Fixture { conn, user_gen: Box::new(Fixture::gen_users(&conn)) }
	}
	
	pub fn create_user(&mut self) -> User {
		self.user_gen.deref_mut()()
	}
	
	pub fn teardown(&self) {
		let tables = vec!["transactions", "accounts", "users"];
		println!("\n--- clean up ---");
		for table in tables {
			diesel::sql_query(format!("DELETE FROM {}", table))
				.execute(&self.conn)
				.map(|n| println!("deleting {} from '{}' table", n, table))
				.expect("deleting db table");
		}
	}
	
	pub fn create_user_and_account(&mut self) -> (User, Account) {
		let user = self.create_user();
		let account = self.create_account(AccountType::Checking, &user);
		(user, account)
	}
	
	pub fn gen_users(conn: &PgConnection) -> impl FnMut() -> User + '_ {
		let input = vec![
			NewUser {
				email: TestUsers::email_vince,
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
		
		let mut iter = input.into_iter();
		move || -> User {
			let new_user = iter.next().expect("consumed all NewUser input");
			diesel::insert_into(users::table)
				.values(new_user)
				.get_result::<User>(conn)
				.unwrap()
		}
	}
}


pub struct Suite<'a> {
	pub user_repo: user::Repo<'a>,
	pub account_repo: account::Repo<'a>,
	pub transaction_repo: transaction::Repo<'a>,
	pub fixture: &'a Fixture<'a>,
}

impl<'a> Suite<'a> {
	pub fn setup(fixture: &'a Fixture) -> Self {
		fixture.teardown();
		
		let conn = &fixture.conn;
		let suite = Suite {
			user_repo: user::Repo::new(conn),
			account_repo: account::Repo::new(conn),
			transaction_repo: transaction::Repo::new(conn),
			fixture,
		};
		
		suite
	}
	
	
	pub fn create_account(&self, account_type: AccountType, user: &User) -> Account {
		let payload = NewAccount {
			user_id: user.id,
			account_type,
		};
		
		diesel::insert_into(accounts::table)
			.values(payload)
			.get_result(&self.fixture.conn)
			.unwrap()
	}
}

#[test]
fn test_suite_setup() {
	let fixture = Fixture::new();
	
	let _suite = Suite::setup(&fixture);
}


