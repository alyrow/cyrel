use std::env;

use anyhow::anyhow;
use celcat::{
    entities::Student,
    fetch::Celcat,
    fetchable::resources::{ResourceList, ResourceListRequest},
};
use dotenv::dotenv;
use sqlx::postgres::PgPool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv();

    let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;
    let celcat = {
        let mut c = Celcat::new("https://services-web.u-cergy.fr/calendar").await?;
        c.login(&env::var("CELCAT_USERNAME")?, &env::var("CELCAT_PASSWORD")?)
            .await?;
        c
    };

    fetch_students(&pool, &celcat).await?;

    Ok(())
}

async fn fetch_students(pool: &PgPool, celcat: &Celcat) -> anyhow::Result<()> {
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
INSERT INTO celcat_students (id, firstname, lastname)
VALUES ( $1, $2, $3 )
ON CONFLICT (id) DO UPDATE SET firstname = EXCLUDED.firstname, lastname = EXCLUDED.lastname
            "#,
            s.id.0.parse::<i64>()?,
            firstname,
            lastname
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
