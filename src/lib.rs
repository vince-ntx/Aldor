#[macro_use]
extern crate diesel;

use std::env;

use diesel::pg::types::sql_types::Uuid;
use diesel::prelude::*;
use diesel::Queryable;
use diesel::sql_types::*;
use serde::{Deserialize, Serialize};
use uuid;

use dotenv::dotenv;
use schema::users;

pub mod schema;
mod error;
// pub mod models;

type Result<T> = std::result::Result<T, error::Error>;

#[derive(Queryable, Debug)]
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
	
	pub fn find_user(&self, key: UserKey<'a>) -> Result<User> {}
}


/// Connect to PostgreSQL database
pub fn create_postgres_connection() -> PgConnection {
	dotenv().ok();
	
	let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
	
	PgConnection::establish(&database_url).expect(&format!("error connecting to {}", database_url))
}
