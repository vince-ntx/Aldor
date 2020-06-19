use std::borrow::Borrow;
use std::collections::HashMap;

use bigdecimal::{BigDecimal, Zero};
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, Table};
use diesel::sql_types::Text;

use bank_api::*;
use bank_api::schema::*;

struct Suite<'a> {
	user_repo: UserRepo<'a>,
	account_repo: AccountRepo<'a>,
	transaction_repo: TransactionRepo<'a>,
	conn: &'a PgConnection,
}

impl<'a> Suite<'a> {
	const user_1_email: &'a str = "vince@gmail";
	
	pub fn get_user() {}
	
	
	pub fn setup(conn: &'a PgConnection) -> Self {
		Suite::teardown(conn);
		
		let mut suite = Suite {
			user_repo: UserRepo::new(conn),
			account_repo: AccountRepo::new(conn),
			transaction_repo: TransactionRepo::new(conn),
			conn,
		};
		
		suite
	}
	
	fn teardown(conn: &PgConnection) {
		let tables = vec!["accounts", "users"];
		println!("\n--- clean up ---");
		for table in tables {
			diesel::sql_query(format!("DELETE FROM {}", table))
				.execute(conn)
				.map(|n| println!("deleting {} from '{}' table", n, table))
				.expect("deleting db table");
		}
	}
	
	fn create_users(&self) -> HashMap<String, User> {
		let mut users_by_email = HashMap::new();
		
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
				users.into_iter().map(|user|
					users_by_email.insert(user.email.clone(), user)
				);
			})
			.expect("inserting users");
		
		users_by_email
	}
	
	
	// fn create_accounts(&self, user_ids: &Vec<uuid::Uuid>) -> Vec<Account> {
	// 	let mut input: Vec<NewAccount> = Vec::new();
	// 	for &id in user_ids {
	// 		input.append(vec![
	// 			NewAccount {
	// 				user_id: id,
	// 				account_type: AccountType::Checking,
	// 				amount: BigDecimal::from(1000),
	// 			},
	// 			NewAccount {
	// 				user_id: id,
	// 				account_type: AccountType::Savings,
	// 				amount: BigDecimal::from(1000),
	// 			}
	// 		].as_mut()
	// 		);
	// 	}
	//
	// 	diesel::insert_into(accounts::table)
	// 		.values(&input)
	// 		.get_results(self.conn)
	// 		.unwrap()
	// }
	
	fn create_account(&self, account_type: AccountType, user_id: uuid::Uuid) -> Account {
		let payload = NewAccount {
			user_id,
			account_type,
			amount: BigDecimal::from(1000),
		};
		
		diesel::insert_into(accounts::table)
			.values(payload)
			.get_result(self.conn)
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
		email: "example@gmail.com",
		first_name: "Tom",
		family_name: "Riddle",
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
	
	let user = users.get(Suite::user_1_email).unwrap();
	
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
		
		assert_eq!(*user, got)
	}
}

#[test]
fn create_account() {
	let conn = get_db_connection();
	let suite = Suite::setup(&conn);
	let users = suite.create_users();
	
	let user = users.get(Suite::user_1_email).unwrap();
	
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
	let users = suite.create_users();
	let user_id = users.get(Suite::user_1_email).unwrap().id;
	
	let mut want = Vec::new();
	let checking = suite.create_account(AccountType::Checking, user_id);
	let savings = suite.create_account(AccountType::Savings, user_id);
	want.push(checking);
	want.push(savings);
	
	
	let got = suite.account_repo.find_accounts(&user_id).unwrap();
	
	assert_eq!(want, got)
}

#[test]
fn account_deposit_and_withdrawal() {
	let conn = get_db_connection();
	let suite = Suite::setup(&conn);
	let users = suite.create_users();
	
	let user_id = users.get(Suite::user_1_email).unwrap().id;
	
	let checking = suite.create_account(AccountType::Checking, user_id);
	
	// deposit
	let deposit_amount = BigDecimal::from(500);
	let got = suite.account_repo.transact(TransactionKey::Deposit, &checking.id, &deposit_amount).unwrap();
	
	let want_amount = (checking.amount) + BigDecimal::from(deposit_amount);
	assert_eq!(got.amount, want_amount, "account's amount should be equal to the deposit");
	
	let withdraw_amount = BigDecimal::from(250);
	let got = suite.account_repo.transact(TransactionKey::Withdraw, &checking.id, &withdraw_amount).unwrap();
	
	let want_amount = (&want_amount) - withdraw_amount;
	assert_eq!(got.amount, want_amount, "account's amount should be equal to (deposit - withdrawal)");
}


#[test]
fn create_transaction() {
	let conn = get_db_connection();
	let suite = Suite::setup(&conn);
	let user_ids = suite.create_users();
	let user_id = user_ids.get(Suite::user_1_email).unwrap().id;
	
	let checking = suite.create_account(AccountType::Checking, user_id);
	
	let to_id = checking.id;
	
	let got = suite.transaction_repo.create(NewTransaction {
		from_id: None,
		to_id: Some(to_id),
		transaction_type: TransactionKey::Deposit,
		amount: BigDecimal::from(250),
	}).unwrap();
	
	let want = Transaction {
		id: got.id,
		from_id: None,
		to_id: Some(to_id),
		transaction_type: TransactionKey::Deposit,
		amount: BigDecimal::from(250),
		timestamp: got.timestamp,
	};
	
	assert_eq!(got, want);
}
