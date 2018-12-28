ALTER TABLE users ADD COLUMN revoke_before TIMESTAMP NOT NULL DEFAULT current_timestamp;
