use chrono::NaiveDateTime;

use super::Course;

pub async fn fetch_calendar(
    start: NaiveDateTime,
    end: NaiveDateTime,
    group: i64,
) -> anyhow::Result<Vec<Course>> {
    todo!()
}
