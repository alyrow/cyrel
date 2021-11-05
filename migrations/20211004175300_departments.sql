CREATE TABLE IF NOT EXISTS departments
(
    id     TEXT PRIMARY KEY,
    name   TEXT NOT NULL UNIQUE,
    domain TEXT NOT NULL
)