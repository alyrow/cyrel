use std::env;

use anyhow::{anyhow, Context};
use celcat::{
    entities::Student,
    fetch::Celcat,
    fetchable::resources::{ResourceList, ResourceListRequest},
};
use dotenv::dotenv;
use sqlx::postgres::PgPool;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init()
        .map_err(|e| anyhow!(e))?;

    let pool = PgPool::connect(&env::var("DATABASE_URL")?)
        .await
        .context("Failed to connect to PostgreSQL")?;
    let celcat = {
        let mut c = Celcat::new("https://services-web.u-cergy.fr/calendar")
            .await
            .context("Failed to connect to Celcat")?;
        c.login(&env::var("CELCAT_USERNAME")?, &env::var("CELCAT_PASSWORD")?)
            .await
            .context("Failed to login to Celcat")?;
        c
    };

    let students: ResourceList<Student> = celcat
        .fetch(ResourceListRequest {
            my_resources: false,
            search_term: "__".to_owned(),
            page_size: 1000000,
            page_number: 0,
            res_type: Student,
        })
        .await?;

    let mut tx = pool.begin().await?;
    for s in students.results {
        let (firstname, lastname) = separate_names(&s.text)?;
        sqlx::query!(
            r#"
INSERT INTO celcat_students (id, firstname, lastname, department)
VALUES ( $1, $2, $3, $4 )
ON CONFLICT (id) DO UPDATE
SET (firstname, lastname, department) = (EXCLUDED.firstname, EXCLUDED.lastname, EXCLUDED.department)
            "#,
            s.id.0.parse::<i64>()?,
            firstname,
            lastname,
            s.dept
        )
        .execute(&mut tx)
        .await?;
    }
    tx.commit().await?;

    Ok(())
}

fn separate_names(name: &str) -> anyhow::Result<(String, String)> {
    let name: String = name
        .split_inclusive(|c: char| !c.is_alphabetic())
        .map(|w| {
            let mut cs = w.chars();
            match cs.next() {
                Some(c) => c.to_string() + &cs.as_str().to_lowercase(),
                None => String::new(),
            }
        })
        .collect();

    match name.rsplit_once(' ') {
        Some((l, f)) => Ok((f.to_owned(), l.to_owned())),
        _ => Err(anyhow!(
            "Can't split '{}' into firstname and lastname",
            name
        )),
    }
}
