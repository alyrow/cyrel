CREATE TABLE IF NOT EXISTS groups
(
    id       SERIAL PRIMARY KEY,
    name     TEXT    NOT NULL,
    referent BIGINT REFERENCES users,
    parent   INTEGER REFERENCES groups,
    private  BOOLEAN NOT NULL
);
