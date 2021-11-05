use crate::models::{Department, User};
use sqlx::PgPool;

pub struct Db {}

impl Db {
    pub async fn match_user(pool: &PgPool, id: i64) -> anyhow::Result<User> {
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
}
