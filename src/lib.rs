#![allow(warnings)]
#[macro_use]
extern crate diesel;


mod schema;
mod account;
mod user;
mod bank_transaction;
mod account_transaction;
mod vault;
mod loan;
mod bank;
mod types;
mod db;

#[cfg(test)]
mod testutil;


