CREATE TABLE users
(
    id           uuid DEFAULT uuid_generate_v4() PRIMARY KEY,
    email        varchar NOT NULL,
    first_name   varchar NOT NULL,
    family_name  varchar NOT NULL,
    phone_number varchar
)