use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

pub use bigdecimal::BigDecimal;
use diesel::PgConnection;
pub use diesel::prelude::*;
use diesel::query_builder::InsertStatement;
use diesel::r2d2::ConnectionManager;
use r2d2::PooledConnection;

use crate::{account, account_transaction, bank_transaction, db, loan, user, vault};
use crate::account::{Account, AccountType, NewAccount};
use crate::schema::{accounts, users, vaults};
use crate::user::{NewUser, User};
use crate::vault::{NewVault, Vault};

pub struct Fixture {
	pub pool: db::PgPool,
	pub user_factory: UserFactory,
	pub account_factory: AccountFactory,
}

impl Fixture {
	pub fn new() -> Self {
		let pool = db::pg_connection();
		let user_factory = UserFactory::new(pool.clone());
		let account_factory = AccountFactory::new(pool.clone());
		Fixture {
			pool,
			user_factory,
			account_factory,
		}
	}
	
	pub fn pool(&self) -> db::PgPool {
		self.pool.clone()
	}
	
	pub fn conn(&self) -> PooledConnection<ConnectionManager<PgConnection>> {
		self.pool.get().unwrap()
	}
	
	
	pub fn insert_main_vault(&self, initial_amount: u32) -> Vault {
		let initial_amount = BigDecimal::from(initial_amount);
		diesel::insert_into(vaults::table)
			.values(NewVault {
				name: "main",
				initial_amount,
			})
			.get_result(&self.conn())
			.unwrap()
	}
	
	pub fn teardown(&self) {
		let tables = vec![
			"loan_payments",
			"loans",
			"account_transactions",
			"bank_transactions",
			"accounts",
			"vaults",
			"users",
		];
		println!("\n--- clean up ---");
		for table in tables {
			diesel::sql_query(format!("DELETE FROM {}", table))
				.execute(&self.conn())
				.map(|n| println!("deleting {} from '{}' table", n, table))
				.expect("deleting db table");
		}
	}
}

pub struct Suite {
	pub user_repo: user::Repo,
	pub account_repo: account::Repo,
	pub vault_repo: vault::Repo,
	pub bank_transaction_repo: bank_transaction::Repo,
	pub account_transaction_repo: account_transaction::Repo,
	pub loan_repo: loan::Repo,
	pub loan_payment_repo: loan::PaymentRepo,
}

impl Suite {
	pub fn setup() -> Self {
		let fixture = Fixture::new();
		fixture.teardown();
		
		let suite = Suite {
			user_repo: user::Repo::new(fixture.pool.clone()),
			account_repo: account::Repo::new(fixture.pool.clone()),
			vault_repo: vault::Repo::new(fixture.pool.clone()),
			bank_transaction_repo: bank_transaction::Repo::new(fixture.pool.clone()),
			account_transaction_repo: account_transaction::Repo::new(fixture.pool.clone()),
			loan_repo: loan::Repo::new(fixture.pool.clone()),
			loan_payment_repo: loan::PaymentRepo::new(fixture.pool.clone()),
		};
		
		suite
	}
}

#[test]
fn test_suite_setup() {
	let _suite = Suite::setup();
}

pub struct UserFactory {
	pool: db::PgPool
}

impl<'a> UserFactory {
	fn new(pool: db::PgPool) -> Self {
		UserFactory { pool }
	}
	
	pub fn defaults() -> NewUser<'a> {
		NewUser {
			email: "default@gmail.com",
			first_name: "Default",
			family_name: "Default",
			phone_number: None,
		}
	}
	
	pub fn user(&self, new_user: NewUser) -> User {
		let conn = self.pool.get().unwrap();
		diesel::insert_into(users::table)
			.values(new_user)
			.get_result::<User>(&conn)
			.unwrap()
	}
	
	pub fn bob(&self) -> User {
		self.user(NewUser {
			email: "bob@gmail.com",
			first_name: "Bob",
			family_name: "Roberts",
			..UserFactory::defaults()
		})
	}
	
	pub fn lucy(&self) -> User {
		self.user(NewUser {
			email: "lucy@gmail.com",
			first_name: "Lucy",
			family_name: "Luke",
			..UserFactory::defaults()
		})
	}
}

pub struct AccountFactory {
	pool: db::PgPool
}

impl<'a> AccountFactory {
	pub fn new(pool: db::PgPool) -> Self {
		AccountFactory { pool }
	}
	
	pub fn checking_account(&self, user_id: uuid::Uuid) -> Account {
		let payload = NewAccount {
			user_id,
			account_type: AccountType::Checking,
		};
		let conn = self.pool.get().unwrap();
		diesel::insert_into(accounts::table)
			.values(payload)
			.get_result(&conn)
			.unwrap()
	}
}


