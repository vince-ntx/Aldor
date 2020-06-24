CREATE TABLE bank_transactions
(
    id               uuid           DEFAULT uuid_generate_v4() PRIMARY KEY,
    account_id       uuid REFERENCES accounts (id)    NOT NULL,
    vault_name       varchar REFERENCES vaults (name) NOT NULL,
    transaction_type varchar                          NOT NULL,
    amount           NUMERIC(12, 4) DEFAULT 0         NOT NULL,
    created_at       timestamptz    DEFAULT NOW()     NOT NULL
);

CREATE TABLE account_transactions
(
    id          uuid           DEFAULT uuid_generate_v4() PRIMARY KEY,
    sender_id   uuid REFERENCES accounts (id) NOT NULL,
    receiver_id uuid REFERENCES accounts (id) NOT NULL,
    amount      NUMERIC(12, 4) DEFAULT 0      NOT NULL,
    created_at  timestamptz    DEFAULT NOW()  NOT NULL
);
