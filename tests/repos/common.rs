use std::collections::HashMap;

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


pub struct Suite<'a> {
	pub user_repo: user::Repo<'a>,
	pub account_repo: account::Repo<'a>,
	pub transaction_repo: transaction::Repo<'a>,
	pub conn: &'a PgConnection,
}

impl<'a> Suite<'a> {
	pub fn setup(conn: &'a PgConnection) -> Self {
		Suite::teardown(conn);
		
		let suite = Suite {
			user_repo: user::Repo::new(conn),
			account_repo: account::Repo::new(conn),
			transaction_repo: transaction::Repo::new(conn),
			conn,
		};
		
		suite
	}
	
	pub fn gen_users(&self) -> impl FnMut() -> User + '_ {
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
			// let x = input.get(idx).unwrap();
			let x = iter.next().expect("bruh");
			diesel::insert_into(users::table)
				.values(x)
				.get_result::<User>(self.conn)
				// .and_then(|u| {
				// 	idx += 1;
				// 	Ok(u)
				// }	)
				.unwrap()
		}
	}
	
	fn teardown(conn: &PgConnection) {
		let tables = vec!["transactions", "accounts", "users"];
		println!("\n--- clean up ---");
		for table in tables {
			diesel::sql_query(format!("DELETE FROM {}", table))
				.execute(conn)
				.map(|n| println!("deleting {} from '{}' table", n, table))
				.expect("deleting db table");
		}
	}
	
	pub fn create_users(&self) -> HashMap<String, User> {
		let mut users_by_email = HashMap::new();
		
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
		
		diesel::insert_into(users::table)
			.values(&input)
			.get_results(self.conn)
			.map(|users: Vec<User>|
				users.into_iter().for_each(
					|u| { users_by_email.insert(u.email.clone(), u); }
				)
			).unwrap();
		
		users_by_email
	}
	
	pub fn create_user(&self) -> User {
		let new_user = NewUser {
			email: "jack@gmail.com",
			first_name: "Jack",
			family_name: "Smith",
			phone_number: None,
		};
		
		diesel::insert_into(users::table)
			.values(&new_user)
			.get_result::<User>(self.conn)
			.unwrap()
	}
	
	
	pub fn create_account(&self, account_type: AccountType, user: &User) -> Account {
		let payload = NewAccount {
			user_id: user.id,
			account_type,
		};
		
		diesel::insert_into(accounts::table)
			.values(payload)
			.get_result(self.conn)
			.unwrap()
	}
	
	pub fn create_user_and_account(&self) -> (User, Account) {
		let user = self.create_user();
		let account = self.create_account(AccountType::Checking, &user);
		(user, account)
	}
}

#[test]
fn test_suite_setup() {
	let conn = get_db_connection();
	let _suite = Suite::setup(&conn);
	
	let mut g = _suite.gen_users();
	let u = g();
	println!("{:?}", u);
	
	let u = g();
	println!("{:?}", u);
	
	
	let u = g();
	println!("{:?}", u);
}


