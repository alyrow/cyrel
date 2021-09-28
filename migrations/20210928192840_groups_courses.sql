CREATE TABLE IF NOT EXISTS groups_courses
    ( group_id INTEGER REFERENCES groups NOT NULL
    , course_id TEXT REFERENCES courses NOT NULL
    );
