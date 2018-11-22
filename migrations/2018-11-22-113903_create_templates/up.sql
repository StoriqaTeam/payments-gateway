CREATE TABLE templates (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    data VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('templates');
