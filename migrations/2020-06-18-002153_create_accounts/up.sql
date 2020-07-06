CREATE TABLE accounts
(
    id           uuid           DEFAULT uuid_generate_v4() PRIMARY KEY,
    user_id      uuid REFERENCES users (id)   NOT NULL,
    account_type varchar                      NOT NULL,
    amount       NUMERIC(12, 4) DEFAULT 0     NOT NULL,
    created_at   timestamptz    DEFAULT NOW() NOT NULL,
    is_open      boolean        DEFAULT true  NOT NULL
)
