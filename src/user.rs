use diesel::PgConnection;
use diesel::prelude::*;

use crate::db;
use crate::schema;
use crate::schema::users;

/// User represents a bank customer
#[derive(Queryable, Identifiable, PartialEq, Debug)]
pub struct User {
	pub id: uuid::Uuid,
	pub email: String,
	pub first_name: String,
	pub family_name: String,
	pub phone_number: Option<String>,
	/* TODO: add additional info here including
	- date of birth
	- home address
	 */
}

/// Data store implementation for operating on users in the database
pub struct Repo {
	db: db::PgPool,
}

impl Repo {
	pub fn new(db: db::PgPool) -> Self {
		Repo { db }
	}
	
	pub fn create(&self, new_user: NewUser) -> db::Result<User> {
		let conn = &self.db.get()?;
		diesel::insert_into(users::table)
			.values(&new_user)
			.get_result(conn)
			.map_err(Into::into)
	}
	
	pub fn find_by_key(&self, key: FindKey) -> db::Result<User> {
		let conn = &self.db.get()?;
		match key {
			FindKey::ID(id) => {
				users::table
					.find(id)
					.first::<User>(conn)
					.map_err(Into::into)
			}
			FindKey::Email(email) => {
				users::table
					.filter(users::email.eq(email))
					.first::<User>(conn)
					.map_err(Into::into)
			}
		}
	}
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
	pub email: &'a str,
	pub first_name: &'a str,
	pub family_name: &'a str,
	pub phone_number: Option<&'a str>,
}

pub enum FindKey<'a> {
	ID(uuid::Uuid),
	Email(&'a str),
}


#[cfg(test)]
mod tests {
	use std::borrow::Borrow;
	
	use crate::testutil::*;
	
	use super::*;
	
	#[test]
	fn insert_user() {
		let fixture = Fixture::new();
		let suite = Suite::setup();
		let user = suite.user_repo.create(NewUser {
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
		let fixture = Fixture::new();
		let user = fixture.user_factory.bob();
		
		let suite = Suite::setup();
		
		let email = user.email.borrow();
		let id = user.id;
		
		// test cases using various FindKeys
		let test_cases = vec![
			FindKey::Email(email),
			FindKey::ID(id)
		];
		
		
		for user_key in test_cases {
			let got = suite.user_repo.find_by_key(user_key)
				.expect("found user");
			
			assert_eq!(user, got)
		}
	}
}
