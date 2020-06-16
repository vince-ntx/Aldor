table! {
    users (id) {
        id -> Uuid,
        email -> Varchar,
        first_name -> Varchar,
        family_name -> Varchar,
        phone_number -> Nullable<Varchar>,
    }
}
