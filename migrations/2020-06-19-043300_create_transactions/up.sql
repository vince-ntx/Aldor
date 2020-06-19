CREATE TABLE transactions
(
    id               uuid           DEFAULT uuid_generate_v4() PRIMARY KEY,
    from_id          uuid REFERENCES accounts (id) NOT NULL,
    to_id            uuid REFERENCES accounts (id) NOT NULL,
    transaction_type varchar                       NOT NULL,
    amount           NUMERIC(12, 4) DEFAULT 0      NOT NULL,
    created_at       timestamp      DEFAULT NOW()  NOT NULL
)
