#![allow(warnings)]

use std::{env, fmt};

use diesel::prelude::*;
use log::*;
use pretty_env_logger;
use warp::Filter;
use warp::filters::log::Info;

use bank_api::*;

#[tokio::main]
async fn main() {
	env::set_var("RUST_LOG", "debug");
	pretty_env_logger::init();
}


