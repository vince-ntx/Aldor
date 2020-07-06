CREATE TABLE loans
(
    id                   uuid           DEFAULT uuid_generate_v4() PRIMARY KEY,
    user_id              uuid REFERENCES users (id)       NOT NULL,
    vault_name           VARCHAR REFERENCES vaults (name) NOT NULL,
    orig_principal       NUMERIC(12, 4)                   NOT NULL,
    balance              NUMERIC(12, 4)                   NOT NULL,
    interest_rate        SMALLINT                         NOT NULL,
    issue_date           date                             NOT NULL,
    maturity_date        date                             NOT NULL,
    payment_frequency    SMALLINT                         NOT NULL,
    compound_frequency   SMALLINT                         NOT NULL,
    accrued_interest     NUMERIC(12, 4) DEFAULT 0         NOT NULL,
    capitalized_interest NUMERIC(12, 4) DEFAULT 0         NOT NULL,
    state                VARCHAR                          NOT NULL
--  created_at  timestamptz    DEFAULT NOW()  NOT NULL
);

CREATE TABLE loan_payments
(
    id                       uuid           DEFAULT uuid_generate_v4() PRIMARY KEY,
    loan_id                  uuid REFERENCES loans (id) NOT NULL,
    principal_due            NUMERIC(12, 4) DEFAULT 0   NOT NULL,
    interest_due             NUMERIC(12, 4) DEFAULT 0   NOT NULL,
    due_date                 date                       NOT NULL,
--     is_paid                  BOOLEAN        DEFAULT FALSE NOT NULL,
    principle_transaction_id uuid REFERENCES bank_transactions (id),
    interest_transaction_id  uuid REFERENCES bank_transactions (id)
);


