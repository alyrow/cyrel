use std::marker::PhantomData;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::celcat::resource::resource_type::{ResourceType, WrapResourceType};

use super::Fetchable;

#[derive(Debug, Serialize, Deserialize)]
#[repr(transparent)]
pub struct CourseId(pub String);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Course {
    pub id: CourseId,
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
    pub all_day: bool,
    pub description: String,
    pub background_color: String,
    pub text_color: String,
    pub departement: Option<String>,    // TODO
    pub faculty: Option<String>,        // TODO
    pub event_category: Option<String>, // TODO
    pub sites: Option<Vec<String>>,     // TODO
    pub modules: Option<Vec<String>>,   // TODO
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CalView {
    Month,
    AgendaWeek,
    AgendaDay,
    ListWeek,
    // TODO: extend this
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarDataRequest<T: ResourceType> {
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
    #[serde(bound(serialize = "T: ResourceType"))]
    pub res_type: WrapResourceType<T>,
    pub cal_view: CalView,
    pub federation_ids: T::Id,
    pub colour_scheme: i64,
}

#[derive(Debug, Deserialize)]
// TODO: serde
pub struct CalendarData<T: ResourceType> {
    courses: Vec<Course>,
    request: PhantomData<T>,
}

impl<T> Fetchable for CalendarData<T>
where
    T: ResourceType,
{
    type Request = CalendarDataRequest<T>;

    const METHOD_NAME: &'static str = "GetCalendarData";
}
