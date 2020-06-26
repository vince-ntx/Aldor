table! {
    account_transactions (id) {
        id -> Uuid,
        sender_id -> Uuid,
        receiver_id -> Uuid,
        amount -> Numeric,
        created_at -> Timestamptz,
    }
}

table! {
    accounts (id) {
        id -> Uuid,
        user_id -> Uuid,
        account_type -> Varchar,
        amount -> Numeric,
        created_at -> Timestamptz,
        is_open -> Bool,
    }
}

table! {
    bank_transactions (id) {
        id -> Uuid,
        account_id -> Uuid,
        vault_name -> Varchar,
        transaction_type -> Varchar,
        amount -> Numeric,
        created_at -> Timestamptz,
    }
}

table! {
    loan_payments (id) {
        id -> Uuid,
        loan_id -> Uuid,
        principal_due -> Numeric,
        interest_due -> Numeric,
        due_date -> Date,
        principle_transaction_id -> Nullable<Uuid>,
        interest_transaction_id -> Nullable<Uuid>,
    }
}

table! {
    loans (id) {
        id -> Uuid,
        user_id -> Uuid,
        vault_name -> Varchar,
        orig_principal -> Numeric,
        balance -> Numeric,
        interest_rate -> Int2,
        issue_date -> Date,
        maturity_date -> Date,
        payment_frequency -> Int2,
        compound_frequency -> Int2,
        accrued_interest -> Numeric,
        capitalized_interest -> Numeric,
        state -> Varchar,
    }
}

table! {
    users (id) {
        id -> Uuid,
        email -> Varchar,
        first_name -> Varchar,
        family_name -> Varchar,
        phone_number -> Nullable<Varchar>,
    }
}

table! {
    vaults (name) {
        name -> Varchar,
        amount -> Numeric,
    }
}

joinable!(accounts -> users (user_id));
joinable!(bank_transactions -> accounts (account_id));
joinable!(bank_transactions -> vaults (vault_name));
joinable!(loan_payments -> loans (loan_id));
joinable!(loans -> users (user_id));
joinable!(loans -> vaults (vault_name));

allow_tables_to_appear_in_same_query!(
    account_transactions,
    accounts,
    bank_transactions,
    loan_payments,
    loans,
    users,
    vaults,
);
