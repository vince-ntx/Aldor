use diesel::PgConnection;
use diesel::prelude::*;

use crate::{db, PgPool, Result};
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

pub struct Repo {
	db: PgPool,
}

impl Repo {
	pub fn new(db: PgPool) -> Self {
		Repo { db }
	}
	
	pub fn create_user(&self, new_user: NewUser) -> db::Result<User> {
		let conn = &self.db.get()?;
		diesel::insert_into(users::table)
			.values(&new_user)
			.get_result(conn)
			.map_err(Into::into)
	}
	
	pub fn find_user(&self, key: FindKey) -> db::Result<User> {
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


