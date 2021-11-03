use crate::models::User;
use sqlx::PgPool;

pub(crate) async fn match_user(pool: &PgPool, id: i64) -> anyhow::Result<User> {
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
