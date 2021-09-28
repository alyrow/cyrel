CREATE TABLE IF NOT EXISTS users_groups
    ( user_id BIGINT REFERENCES users NOT NULL
    , group_id INTEGER REFERENCES groups NOT NULL
    );
