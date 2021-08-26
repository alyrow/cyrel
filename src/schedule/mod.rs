use chrono::{DateTime, Utc};
use serde::Serialize;

pub mod celcat;

#[derive(Debug, Serialize)]
pub struct Course {
    pub uid: String,
    pub start: DateTime<Utc>, // FIXME: UTC or local?
    pub end: DateTime<Utc>,
    pub category: String,
    pub name: String,
    pub location: String,
    pub prof: String,
}
