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
}
