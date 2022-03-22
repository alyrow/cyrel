ALTER TABLE users_groups
ADD UNIQUE (user_id, group_id);

ALTER TABLE groups_courses
ADD UNIQUE (group_id, course_id);

ALTER TABLE clients_users_config
ADD UNIQUE (user_id, client_id);
