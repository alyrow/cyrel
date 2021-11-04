CREATE TABLE departments
(
    id     TEXT PRIMARY KEY NOT NULL,
    name   TEXT             NOT NULL UNIQUE,
    domain TEXT             NOT NULL
)