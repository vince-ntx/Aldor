use diesel::PgConnection;
use diesel::prelude::*;

use crate::{Result, User};
use crate::schema;
use crate::schema::users;

pub struct Repo<'a> {
	db: &'a PgConnection,
}

impl<'a> Repo<'a> {
	pub fn new(db: &'a PgConnection) -> Self {
		Repo { db }
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

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
	pub email: &'a str,
	pub first_name: &'a str,
	pub family_name: &'a str,
	pub phone_number: Option<&'a str>,
}

pub enum UserKey<'a> {
	ID(uuid::Uuid),
	Email(&'a str),
}


