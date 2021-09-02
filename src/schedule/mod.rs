use chrono::NaiveDateTime;
use serde::Serialize;

pub mod celcat;

#[derive(Debug, Serialize)]
pub struct Course {
    pub id: String,
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
    pub category: Option<String>,
    pub description: Vec<String>,
}
