use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

pub use bigdecimal::BigDecimal;
use diesel::PgConnection;
pub use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use r2d2::PooledConnection;

pub use bank_api::*;
use bank_api::AccountType::Checking;

pub struct TestUsers {}

impl<'a> TestUsers {
	pub const email_vince: &'a str = "vince@gmail.com";
	pub const email_jack: &'a str = "jack@gmail.com";
}

pub struct Fixture<'a> {
	pub pool: PgPool,
	user_gen: Box<dyn FnMut() -> User + 'a>,
}

impl<'a> Fixture<'_> {
	pub fn new() -> Self {
		let pool = get_db_connection();
		let pool_clone = pool.clone();
		Fixture {
			pool,
			user_gen: Box::new(Fixture::gen_users(pool_clone)),
		}
	}
	
	pub fn conn(&self) -> PooledConnection<ConnectionManager<PgConnection>> {
		self.pool.get().unwrap()
	}
	
	pub fn create_user(&mut self) -> User {
		self.user_gen.deref_mut()()
	}
	
	pub fn create_checking_account(&self, user: &User) -> Account {
		let payload = NewAccount {
			user_id: user.id,
			account_type: AccountType::Checking,
		};
		
		diesel::insert_into(accounts::table)
			.values(payload)
			.get_result(&self.conn())
			.unwrap()
	}
	
	pub fn teardown(&self) {
		let tables = vec!["transactions", "accounts", "users"];
		println!("\n--- clean up ---");
		for table in tables {
			diesel::sql_query(format!("DELETE FROM {}", table))
				.execute(&self.conn())
				.map(|n| println!("deleting {} from '{}' table", n, table))
				.expect("deleting db table");
		}
	}
	
	pub fn create_user_and_account(&mut self) -> (User, Account) {
		let user = self.create_user();
		let account = self.create_checking_account(&user);
		(user, account)
	}
	
	pub fn gen_users(pool: PgPool) -> impl FnMut() -> User + 'static {
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
			let conn = pool.get().unwrap();
			let new_user = iter.next().expect("consumed all NewUser input");
			diesel::insert_into(users::table)
				.values(new_user)
				.get_result::<User>(&conn)
				.unwrap()
		}
	}
}


pub struct Suite {
	pub user_repo: user::Repo,
	pub account_repo: account::Repo,
	pub transaction_repo: transaction::Repo,
}

impl Suite {
	pub fn setup() -> Self {
		let fixture = Fixture::new();
		fixture.teardown();
		
		let suite = Suite {
			user_repo: user::Repo::new(fixture.pool.clone()),
			account_repo: account::Repo::new(fixture.pool.clone()),
			transaction_repo: transaction::Repo::new(fixture.pool.clone()),
		};
		
		suite
	}
}

#[test]
fn test_suite_setup() {
	let _suite = Suite::setup();
}


