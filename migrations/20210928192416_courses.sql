CREATE TABLE IF NOT EXISTS courses
    ( id TEXT PRIMARY KEY
    , start_time TIMESTAMP WITHOUT TIME ZONE NOT NULL
    , end_time TIMESTAMP WITHOUT TIME ZONE
    , category TEXT
    , module TEXT
    , room TEXT
    , teacher TEXT
    , description TEXT
    );
