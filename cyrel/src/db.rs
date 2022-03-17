use chrono::NaiveDateTime;
use sqlx::PgPool;
use tracing::{error, info, warn};

use crate::models::{Department, Group, User};
use crate::rpc::RpcError;
use crate::schedule::Course;

pub struct Db {}

impl Db {
    pub async fn match_user_by_id(pool: &PgPool, id: i64) -> anyhow::Result<User> {
        let user = sqlx::query!(
            r#"
SELECT id, firstname, lastname, email, password
FROM users
WHERE id = $1
        "#,
            id
        )
        .fetch_one(pool)
        .await?;

        Ok(User {
            id,
            firstname: user.firstname,
            lastname: user.lastname,
            email: user.email,
            password: user.password,
        })
    }

    pub async fn match_user_by_email(pool: &PgPool, email: String) -> anyhow::Result<User> {
        let user = sqlx::query!(
            r#"
SELECT id, firstname, lastname, email, password
FROM users
WHERE email = $1
        "#,
            email
        )
        .fetch_one(pool)
        .await?;

        Ok(User {
            id: user.id,
            firstname: user.firstname,
            lastname: user.lastname,
            email: user.email,
            password: user.password,
        })
    }

    pub async fn match_department(pool: &PgPool, id: String) -> anyhow::Result<Department> {
        let dep = sqlx::query!(
            r#"
SELECT id, name, domain
FROM departments
WHERE id = $1
        "#,
            id
        )
        .fetch_one(pool)
        .await?;

        Ok(Department {
            id,
            name: dep.name,
            domain: dep.domain,
        })
    }

    pub async fn match_celcat_student(
        pool: &PgPool,
        id: i64,
        department: String,
    ) -> anyhow::Result<(String, String)> {
        let student = sqlx::query!(
            r#"
SELECT firstname, lastname
FROM celcat_students
WHERE id = $1 AND department = $2
        "#,
            id,
            department
        )
        .fetch_one(pool)
        .await?;

        Ok((student.firstname, student.lastname))
    }

    pub async fn insert_user(pool: &PgPool, user: User) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;
        let student = sqlx::query!(
            r#"
INSERT INTO users (id, firstname, lastname, email, password)
VALUES ($1, $2, $3, $4, $5)
        "#,
            user.id,
            user.firstname,
            user.lastname,
            user.email,
            user.password
        )
        .execute(&mut tx)
        .await?;
        tx.commit().await?;

        Ok(())
    }

    pub async fn match_group(pool: &PgPool, group: i32) -> anyhow::Result<Group> {
        let grp = sqlx::query!(
            r#"
SELECT id, name, referent, parent, private
FROM groups
WHERE id = $1
        "#,
            group
        )
        .fetch_one(pool)
        .await?;

        Ok(Group {
            id: grp.id,
            name: grp.name,
            referent: grp.referent,
            parent: grp.parent,
            private: grp.private,
        })
    }

    pub async fn is_user_in_group(
        pool: &PgPool,
        user_id: i64,
        group_id: i32,
    ) -> anyhow::Result<()> {
        let grp = sqlx::query!(
            r#"
SELECT user_id, group_id
FROM users_groups
WHERE user_id = $1 AND group_id = $2
        "#,
            user_id,
            group_id
        )
            .fetch_one(pool)
            .await?;

        Ok(())
    }

    pub async fn is_user_in_group_or_brother_group(
        pool: &PgPool,
        user_id: i64,
        group_id: i32,
    ) -> anyhow::Result<()> {
        let group = {
            let result = Db::is_user_in_group(pool, user_id, group_id).await;
            match result {
                Ok(_) => return Ok(()),
                Err(err) => {
                    ()
                }
            }
        };

        let bro = sqlx::query!(
            r#"
SELECT id, name, referent, parent, private
FROM groups
WHERE id = $1 AND parent IS NOT NULL
        "#,
            group_id
        )
            .fetch_one(pool)
            .await?;

        Db::is_user_in_group(pool, user_id, bro.parent.expect("Parent id can't be null")).await
    }

    pub async fn insert_user_in_group(
        pool: &PgPool,
        user_id: i64,
        group_id: i32,
    ) -> anyhow::Result<()> {
        let user: User = {
            let result = Db::match_user_by_id(pool, user_id).await;
            match result {
                Ok(user) => user,
                Err(err) => {
                    warn!("{}", err.to_string());
                    return Err(RpcError::Unimplemented.into());
                }
            }
        };
        let group: Group = {
            let result = Db::match_group(pool, group_id).await;
            match result {
                Ok(grp) => grp,
                Err(err) => {
                    warn!("{}", err.to_string());
                    return Err(RpcError::Unimplemented.into());
                }
            }
        };
        if group.private {
            return Err(RpcError::Unimplemented.into());
        }
        let result = Db::is_user_in_group(pool, user.id, group.id).await;
        match result {
            Ok(_) => {
                warn!(
                    "user {} is already in group {} ({})",
                    user.id, group.id, group.name
                );
                return Err(RpcError::Unimplemented.into());
            }
            Err(_) => {}
        }

        let mut tx = pool.begin().await?;
        let group_add = sqlx::query!(
            r#"
INSERT INTO users_groups (user_id, group_id)
VALUES ($1, $2)
        "#,
            user.id,
            group.id
        )
        .execute(&mut tx)
        .await?;
        tx.commit().await?;

        Ok(())
    }

    pub async fn get_user_groups(pool: &PgPool, user_id: i64) -> anyhow::Result<Vec<Group>> {
        let grps = sqlx::query!(
            r#"
SELECT user_id, group_id
FROM users_groups
WHERE user_id = $1
        "#,
            user_id
        )
        .fetch_all(pool)
        .await?;

        let mut groups = Vec::<Group>::new();

        for grp in grps {
            let group = Db::match_group(pool, grp.group_id).await?;
            groups.push(group)
        }

        Ok(groups)
    }

    pub async fn get_all_groups(pool: &PgPool, user_id: i64) -> anyhow::Result<Vec<Group>> {
        let grps = sqlx::query!(
            r#"
SELECT id, name, referent, parent, private
FROM groups
WHERE private = false
        "#
        )
        .fetch_all(pool)
        .await?;

        let mut groups = Vec::<Group>::new();

        for grp in grps {
            let group = Group {
                id: grp.id,
                name: grp.name,
                referent: grp.referent,
                parent: grp.parent,
                private: grp.private,
            };
            groups.push(group)
        }

        Ok(groups)
    }

    pub async fn get_group_courses(
        pool: &PgPool,
        group_id: i32,
        start: NaiveDateTime,
        end: NaiveDateTime,
    ) -> anyhow::Result<Vec<Course>> {
        let courses_id = sqlx::query!(
            r#"
SELECT course_id
FROM groups_courses
WHERE group_id = $1
        "#,
            group_id
        )
        .fetch_all(pool)
        .await?;

        let mut courses = Vec::<Course>::new();

        for course_id in courses_id {
            let course = sqlx::query!(
                r#"
SELECT id, start_time, end_time, category, module, room, teacher, description
FROM courses
WHERE id = $1
        "#,
                course_id.course_id
            )
            .fetch_one(pool)
            .await?;

            let course = Course {
                id: course.id,
                start: course.start_time,
                end: course.end_time,
                category: course.category,
                module: course.module,
                room: course.room,
                teacher: course.teacher,
                description: course.description,
            };

            if course.start >= start && course.start <= end {
                courses.push(course);
            }
        }

        Ok(courses)
    }

    pub async fn is_client_exist(
        pool: &PgPool,
        client_id: i32,
    ) -> anyhow::Result<()> {
        let _ = sqlx::query!(
            r#"
SELECT id, name
FROM clients
WHERE id = $1
        "#,
            client_id
        )
            .fetch_one(pool)
            .await?;


        Ok(())
    }

    pub async fn get_client_user_config(
        pool: &PgPool,
        client_id: i32,
        user_id: i64,
    ) -> anyhow::Result<Option<String>> {
        let config = sqlx::query!(
            r#"
SELECT config
FROM clients_users_config
WHERE client_id = $1 AND user_id = $2
        "#,
            client_id,
            user_id
        )
            .fetch_one(pool)
            .await?;


        Ok(config.config)
    }

    pub async fn set_client_user_config(
        pool: &PgPool,
        client_id: i32,
        user_id: i64,
        config: String,
    ) -> anyhow::Result<()> {
        let user: User = {
            let result = Db::match_user_by_id(pool, user_id).await;
            match result {
                Ok(user) => user,
                Err(err) => {
                    warn!("{}", err.to_string());
                    return Err(RpcError::Unimplemented.into());
                }
            }
        };

        let mut tx = pool.begin().await?;

        let result = Db::get_client_user_config(pool, client_id, user.id).await;
        match result {
            Ok(_) => {
                let _ = sqlx::query!(
                    r#"
UPDATE clients_users_config
SET config = $3
WHERE client_id = $1 AND user_id = $2
                "#,
                    client_id,
                    user.id,
                    config
                )
                    .execute(&mut tx)
                    .await?;
            },
            Err(_) => {
                let _ = sqlx::query!(
            r#"
INSERT INTO clients_users_config (client_id, user_id, config)
VALUES ($1, $2, $3)
                "#,
                    client_id,
                    user.id,
                    config
                )
                    .execute(&mut tx)
                    .await?;
            }
        }
        tx.commit().await?;

        Ok(())
    }

    pub async fn update_user(pool: &PgPool, user: User) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;
        let student = sqlx::query!(
            r#"
UPDATE users
SET firstname = $2, lastname = $3, email = $4, password = $5
WHERE id = $1
        "#,
            user.id,
            user.firstname,
            user.lastname,
            user.email,
            user.password
        )
            .execute(&mut tx)
            .await?;
        tx.commit().await?;

        Ok(())
    }
}
