CREATE TABLE transactions_fiat (
    id UUID PRIMARY KEY,
    fiat_value VARCHAR NOT NULL,
    fiat_currency VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('transactions_fiat');
