CREATE TABLE IF NOT EXISTS clients_users_config
(
    user_id   BIGINT REFERENCES users    NOT NULL,
    client_id INTEGER REFERENCES clients NOT NULL,
    config    TEXT
);