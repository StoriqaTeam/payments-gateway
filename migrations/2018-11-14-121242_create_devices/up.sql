CREATE TABLE devices (
    device_id VARCHAR NOT NULL,
    device_os VARCHAR NOT NULL,
    user_id INTEGER NOT NULL REFERENCES users,
    public_key VARCHAR NOT NULL,
    last_timestamp BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

ALTER TABLE devices ADD PRIMARY KEY (device_id, user_id);

SELECT diesel_manage_updated_at('devices');
