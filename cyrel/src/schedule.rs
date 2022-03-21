use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Course {
    /// Unique ID
    pub id: String,
    pub start: NaiveDateTime,
    pub end: Option<NaiveDateTime>,
    pub category: Option<String>,

    /// Subject being taught
    pub module: Option<String>,
    pub room: Option<String>,
    pub teacher: Option<String>,

    /// Any additional description
    pub description: Option<String>,
}
