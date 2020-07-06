Aldor: Banking infrastructure written in Rust
---------
### Core Features
- Deposit or withdraw funds from the bank 
- Allow users to transfer funds to one another 
- Initiate and handle amortized bank loans and repayments
- Manage user accounts and transaction data

### Setup 
1. Clone this repository and run `cargo build`
1. Install PostgreSQL 9.5+
1. Install the [Diesel CLI](http://diesel.rs/guides/getting-started/) using `cargo install diesel_cli` 
1. Set your PostgreSQL database info by adding a `DATBASE_URL` environment variable. You can define it in `.env`
    - Ex) `DATABASE_URL = postgres://username:password@localhost/db_name`
1. Run `diesel setup` to create the database and all tables 
1. Test the database connection in the app by running `cargo test connection` 

### Todo
- Calculate and store savings and loan profits for the bank
- Expose API through a web api 



