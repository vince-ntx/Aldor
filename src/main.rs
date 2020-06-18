use std::{env, fmt};

use diesel::prelude::*;
use log::*;
use pretty_env_logger;
use warp::Filter;
use warp::filters::log::Info;

use bank_api::*;

use crate::schema::users::dsl;

#[tokio::main]
async fn main() {
	use dsl::*;
	env::set_var("RUST_LOG", "debug");
	pretty_env_logger::init();
	
	let conn = connection();
	
	let userRepo = UserRepo::new(&conn);
	
	let new_user = userRepo.create_user(NewUser {
		email: "vince@gmail.com",
		first_name: "vince",
		family_name: "xiao",
		phone_number: Some("555-5555"),
	});
	
	// let results = users.limit(5).load::<User>(&conn)
	// 	.expect("error loading users");
	
	
	// let log = warp::log::custom(|info: Info| {
	// 	info!(
	// 		target: "bank::api",
	// 		"\"{} {} {:?}\" \t{} {} {:?}",
	// 		info.method(),
	// 		info.path(),
	// 		info.version(),
	// 		info.status().canonical_reason().unwrap_or_else(|| "-"),
	// 		info.status().as_u16(),
	// 		info.elapsed(),
	// 	);
	// });
	// let routes = warp::any().map(|| "Hello world!").with(log);
	// warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}


