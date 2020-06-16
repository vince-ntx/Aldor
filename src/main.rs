use std::{env, fmt};

use log::*;
use pretty_env_logger;
use warp::Filter;
use warp::filters::log::Info;

#[tokio::main]
async fn main() {
	env::set_var("RUST_LOG", "debug");
	pretty_env_logger::init();
	
	let log = warp::log::custom(|info: Info| {
		info!(
			target: "bank::api",
			"\"{} {} {:?}\" \t{} {} {:?}",
			info.method(),
			info.path(),
			info.version(),
			info.status().canonical_reason().unwrap_or_else(|| "-"),
			info.status().as_u16(),
			info.elapsed(),
		);
	});
	let routes = warp::any().map(|| "Hello world!").with(log);
	warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}


