CREATE TABLE devices_tokens (
    id UUID PRIMARY KEY,
    device_id VARCHAR NOT NULL,
    device_os VARCHAR NOT NULL,
    user_id INTEGER NOT NULL REFERENCES users,
    public_key VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('devices_tokens');
