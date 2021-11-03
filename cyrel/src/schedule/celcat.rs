use chrono::NaiveDateTime;
use futures::future::try_join_all;

use super::Course;

pub async fn fetch_calendar(
    start: NaiveDateTime,
    end: NaiveDateTime,
    group: i64,
) -> anyhow::Result<Vec<Course>> {
    todo!()
}
