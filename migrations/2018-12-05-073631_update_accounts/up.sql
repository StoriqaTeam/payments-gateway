ALTER TABLE accounts ADD CONSTRAINT fk_users FOREIGN KEY (user_id) REFERENCES users (id);
