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
    transactions (id) {
        id -> Uuid,
        from_id -> Nullable<Uuid>,
        to_id -> Nullable<Uuid>,
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

joinable!(accounts -> users (user_id));

allow_tables_to_appear_in_same_query!(
    accounts,
    transactions,
    users,
);
