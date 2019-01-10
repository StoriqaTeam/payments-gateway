CREATE TABLE transactions_fiat (
    id UUID PRIMARY KEY,
    fiat_value VARCHAR,
    fiat_currency VARCHAR,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('transactions_fiat');
