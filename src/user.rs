use diesel::PgConnection;
use diesel::prelude::*;

use crate::Result;
use crate::schema;
use crate::schema::users;

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

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
	pub email: &'a str,
	pub first_name: &'a str,
	pub family_name: &'a str,
	pub phone_number: Option<&'a str>,
}

pub struct UserRepo<'a> {
	db: &'a PgConnection,
}

pub enum UserKey<'a> {
	ID(uuid::Uuid),
	Email(&'a str),
}

impl<'a> UserRepo<'a> {
	pub fn new(db: &'a PgConnection) -> Self {
		UserRepo { db }
	}
	
	pub fn create_user(&self, new_user: NewUser) -> Result<User> {
		use schema::users::dsl::*;
		diesel::insert_into(users)
			.values(&new_user)
			.get_result(self.db)
			.map_err(Into::into)
	}
	
	pub fn find_user(&self, key: UserKey<'a>) -> Result<User> {
		match key {
			UserKey::ID(id) => {
				users::table
					.find(id)
					.first::<User>(self.db)
					.map_err(Into::into)
			}
			UserKey::Email(email) => {
				users::table
					.filter(users::email.eq(email))
					.first::<User>(self.db)
					.map_err(Into::into)
			}
		}
	}
}



