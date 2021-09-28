CREATE TABLE IF NOT EXISTS users       
    ( id BIGINT PRIMARY KEY
    , firstname TEXT NOT NULL
    , lastname TEXT NOT NULL
    , email TEXT NOT NULL UNIQUE
    , password TEXT NOT NULL
    );
