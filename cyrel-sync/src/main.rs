use std::env;

use anyhow::{anyhow, Context};
use celcat::{
    entities::{Student, StudentId},
    fetch::Celcat,
    fetchable::{
        calendar::{CalView, CalendarData, CalendarDataRequest, Course},
        event::{Element, Event, EventRequest, RawElement},
        resources::{ResourceList, ResourceListRequest},
    },
};
use chrono::naive::NaiveDate;
use dotenv::dotenv;
use futures::future::try_join_all;
use log::error;
use sqlx::postgres::PgPool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let _ = dotenv();

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

    update_students(&pool, &celcat)
        .await
        .context("Failed to update student list")?;

    let gr = get_group_referents(&pool)
        .await
        .context("Failed to get groups referents")?;
    try_join_all(
        gr.into_iter()
            .map(|(g, r)| update_courses(&pool, &celcat, g, r)),
    )
    .await
    .context("Failed to update courses")?;

    Ok(())
}

async fn update_students(pool: &PgPool, celcat: &Celcat) -> anyhow::Result<()> {
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
SET (firstname, lastname) = (EXCLUDED.firstname, EXCLUDED.lastname)
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

async fn get_group_referents(pool: &PgPool) -> anyhow::Result<Vec<(i32, StudentId)>> {
    let referents = sqlx::query!(
        r#"
SELECT id, referent
FROM groups
WHERE referent IS NOT NULL
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(referents
        .into_iter()
        .map(|r| {
            (
                r.id,
                StudentId(
                    r.referent
                        .expect("the query should not return any null values")
                        .to_string(),
                ),
            )
        })
        .collect())
}

async fn update_courses(
    pool: &PgPool,
    celcat: &Celcat,
    group: i32,
    referent: StudentId,
) -> anyhow::Result<()> {
    let calendar: CalendarData<Student> = celcat
        .fetch(CalendarDataRequest {
            start: NaiveDate::from_ymd(2021, 9, 1).and_hms(0, 0, 0),
            end: NaiveDate::from_ymd(2022, 9, 1).and_hms(0, 0, 0),
            res_type: Student,
            cal_view: CalView::Month,
            federation_ids: referent,
            colour_scheme: 3,
        })
        .await?;

    sqlx::query!(
        r#"
DELETE FROM groups_courses
WHERE group_id = $1
        "#,
        group
    )
    .execute(pool)
    .await?;

    try_join_all(
        calendar
            .courses
            .iter()
            .map(|c| update_course(pool, celcat, group, c)),
    )
    .await
    .with_context(|| format!("Failed to update courses for group {}", group))?;

    Ok(())
}

async fn update_course(
    pool: &PgPool,
    celcat: &Celcat,
    group: i32,
    course: &Course,
) -> anyhow::Result<()> {
    let event: Event = match celcat
        .fetch(EventRequest {
            event_id: course.id.clone(),
        })
        .await
    {
        Ok(event) => event,
        Err(err) => {
            error!(
                "Failed to fetch side bar event for course {}: {}",
                course.id.0, err
            );
            return Ok(());
        }
    };

    let mut category: Option<String> = None;
    let mut module: Option<String> = None;
    let mut room: Option<String> = None;
    let mut teacher: Option<String> = None;
    let mut description: Option<String> = None;

    for e in event.elements.0 {
        use Element::*;
        match e {
            Category(RawElement { content, .. }) => {
                category = content;
            }
            Module(RawElement { content, .. }) => {
                module = content;
            }
            Room(RawElement { content, .. }) => {
                room = content;
            }
            Teacher(RawElement { content, .. }) => {
                teacher = content;
            }
            Name(RawElement { content, .. }) => {
                description = content;
            }
            _ => {}
        }
    }

    let mut tx = pool.begin().await?;
    sqlx::query!(
        r#"
DELETE FROM courses
WHERE id = $1
        "#,
        course.id.0
    )
    .execute(&mut tx)
    .await?;

    sqlx::query!(
        r#"
INSERT INTO courses
    ( id
    , start_time
    , end_time
    , category
    , module
    , room
    , teacher
    , description
    )
VALUES ( $1, $2, $3, $4, $5, $6, $7, $8 )
        "#,
        course.id.0,
        course.start,
        course.end,
        category,
        module,
        room,
        teacher,
        description
    )
    .execute(&mut tx)
    .await?;

    sqlx::query!(
        r#"
INSERT INTO groups_courses (group_id, course_id)
VALUES ( $1, $2 )
        "#,
        group,
        course.id.0
    )
    .execute(&mut tx)
    .await?;
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
