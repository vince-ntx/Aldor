table! {
    account_transactions (id) {
        id -> Uuid,
        sender_id -> Uuid,
        receiver_id -> Uuid,
        amount -> Numeric,
        created_at -> Timestamp,
    }
}

table! {
    accounts (id) {
        id -> Uuid,
        user_id -> Uuid,
        account_type -> Varchar,
        amount -> Numeric,
        created_at -> Timestamp,
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
        created_at -> Timestamp,
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

allow_tables_to_appear_in_same_query!(
    account_transactions,
    accounts,
    bank_transactions,
    users,
    vaults,
);
