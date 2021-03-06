use std::env;
use std::sync::Arc;
use std::{collections::HashMap, time::Duration};

use anyhow::{anyhow, Context};
use celcat::{
    entities::{Student, StudentId},
    fetch::Celcat,
    fetchable::{
        calendar::{CalView, CalendarData, CalendarDataRequest, Course},
        event::{Element, Event, EventRequest, RawElement},
    },
};
use chrono::naive::NaiveDate;
use dotenv::dotenv;
use futures::future::{join_all, try_join_all};
use sqlx::postgres::PgPool;
use tokio::sync::{mpsc, oneshot, Mutex, RwLock};
use tokio_retry::{strategy::ExponentialBackoff, Retry};
use tracing::{error, warn};
use tracing_subscriber::EnvFilter;

struct State {
    pool: PgPool,
    celcat: Celcat,
}

type Message = (Course, oneshot::Sender<()>);

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

    let state: &_ = Box::leak(Box::new(State { pool, celcat }));

    let gr = get_group_referents(&state.pool)
        .await
        .context("Failed to get groups referents")?;

    let (tx, rx) = mpsc::channel(100);
    let tx_ref = &tx;

    let handle = tokio::spawn(async move { event_updater(state, rx).await });

    join_all(gr.into_iter().map(|(g, r)| async move {
        if let Err(err) = update_courses(&state, g, r, tx_ref.clone()).await {
            error!("Failed to update courses: {}", err);
        }
    }))
    .await;

    drop(tx);
    handle.await?;

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
    state: &State,
    group: i32,
    referent: StudentId,
    s: mpsc::Sender<Message>,
) -> anyhow::Result<()> {
    let calendar: CalendarData<Student> = state
        .celcat
        .fetch(CalendarDataRequest {
            start: NaiveDate::from_ymd(2021, 9, 1).and_hms(0, 0, 0),
            end: NaiveDate::from_ymd(2022, 9, 1).and_hms(0, 0, 0),
            res_type: Student,
            cal_view: CalView::Month,
            federation_ids: referent,
            colour_scheme: 3,
        })
        .await?;

    let mut tx = state.pool.begin().await?;

    sqlx::query!(
        r#"
DELETE FROM groups_courses
WHERE group_id = $1
        "#,
        group
    )
    .execute(&mut tx)
    .await?;

    let tx = Mutex::new(tx);

    try_join_all(
        calendar
            .courses
            .iter()
            .map(|c| update_course(&tx, group, c, s.clone())),
    )
    .await
    .with_context(|| format!("Failed to update courses for group {}", group))?;

    tx.into_inner().commit().await?;

    Ok(())
}

async fn update_course(
    tx: &Mutex<sqlx::Transaction<'static, sqlx::Postgres>>,
    group: i32,
    course: &Course,
    s: mpsc::Sender<Message>,
) -> anyhow::Result<()> {
    let (otx, orx) = oneshot::channel();
    s.send((course.clone(), otx)).await?;
    if let Err(_) = orx.await {
        return Err(anyhow!("Failed to update side bar event"));
    }

    let mut tx = tx.lock().await;
    sqlx::query!(
        r#"
INSERT INTO groups_courses (group_id, course_id)
VALUES ( $1, $2 )
        "#,
        group,
        course.id.0
    )
    .execute(&mut *tx)
    .await?;

    Ok(())
}

async fn event_updater(state: &'static State, mut rx: mpsc::Receiver<Message>) {
    let mut already_updated = HashMap::<String, Arc<RwLock<()>>>::new();

    while let Some((c, s)) = rx.recv().await {
        if let Some(lock) = already_updated.get(&c.id.0) {
            let lock = Arc::clone(lock);
            tokio::spawn(async move {
                lock.read().await;
                if let Err(_) = s.send(()) {
                    warn!("The receiver dropped");
                }
            });
        } else {
            let lock = Arc::new(RwLock::new(()));
            let pending = Arc::clone(&lock).write_owned().await;
            already_updated.insert(c.id.0.clone(), lock);

            tokio::spawn(async move {
                let strategy =
                    ExponentialBackoff::from_millis(100).max_delay(Duration::from_secs(60));

                let updated = Retry::spawn(strategy, || update_event(state, c.clone())).await;
                if let Err(err) = updated {
                    // BUG: course id is in `already_updated` but we failed to update it.
                    error!("Failed to update side bar event: {}", err);
                    return;
                }
                drop(pending);
                if let Err(_) = s.send(()) {
                    warn!("The receiver dropped");
                }
            });
        }
    }
}

async fn update_event(state: &State, course: Course) -> anyhow::Result<()> {
    let event: Event = match state
        .celcat
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
ON CONFLICT (id) DO UPDATE
SET ( start_time
    , end_time
    , category
    , module
    , room
    , teacher
    , description
    ) = ( EXCLUDED.start_time
        , EXCLUDED.end_time
        , EXCLUDED.category
        , EXCLUDED.module
        , EXCLUDED.room
        , EXCLUDED.teacher
        , EXCLUDED.description
        )
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
    .execute(&state.pool)
    .await?;

    Ok(())
}
