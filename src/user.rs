use diesel::PgConnection;
use diesel::prelude::*;

use crate::{PgPool, Result, User};
use crate::schema;
use crate::schema::users;

pub struct Repo {
	db: PgPool,
}

impl Repo {
	pub fn new(db: PgPool) -> Self {
		Repo { db }
	}
	
	pub fn create_user(&self, new_user: NewUser) -> Result<User> {
		let conn = &self.db.get()?;
		diesel::insert_into(users::table)
			.values(&new_user)
			.get_result(conn)
			.map_err(Into::into)
	}
	
	pub fn find_user(&self, key: UserKey) -> Result<User> {
		let conn = &self.db.get()?;
		match key {
			UserKey::ID(id) => {
				users::table
					.find(id)
					.first::<User>(conn)
					.map_err(Into::into)
			}
			UserKey::Email(email) => {
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

pub enum UserKey<'a> {
	ID(uuid::Uuid),
	Email(&'a str),
}


