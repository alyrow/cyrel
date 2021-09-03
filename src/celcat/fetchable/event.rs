use serde::{Deserialize, Serialize};

use crate::celcat::entity::entity_type::{EntityTypeTrait, WrapEntityType};
use crate::celcat::entity::{entity_type, UnknownId};
use crate::celcat::resource::resource_type;

use super::calendar::CourseId;
use super::Fetchable;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SideBarEvent {
    pub federation_id: UnknownId,
    pub entity_type: WrapEntityType<entity_type::Unknown>,
    pub elements: Vec<SideBarEventElement>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawSideBarEventElement<T: EntityTypeTrait> {
    pub content: Option<String>,
    #[serde(bound(deserialize = "T: EntityTypeTrait"))]
    pub federation_id: T::Id,
    #[serde(bound(deserialize = "T: EntityTypeTrait"))]
    pub entity_type: WrapEntityType<T>,
    pub assignment_context: Option<String>,
    pub contains_hyperlinks: bool,
    pub is_notes: bool,
    pub is_student_specific: bool,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "label")]
pub enum SideBarEventElement {
    Time(RawSideBarEventElement<entity_type::Unknown>),
    #[serde(rename = "Catégorie")]
    Category(RawSideBarEventElement<entity_type::Unknown>),
    #[serde(rename = "Matière")]
    Module(RawSideBarEventElement<resource_type::Module>),
    #[serde(rename = "Salle")]
    Room(RawSideBarEventElement<resource_type::Room>),
    #[serde(rename = "Enseignant")]
    Teacher(RawSideBarEventElement<resource_type::Teacher>),
    #[serde(rename = "Notes")]
    Grades(RawSideBarEventElement<entity_type::Unknown>),
    Name(RawSideBarEventElement<entity_type::Unknown>),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SideBarEventRequest {
    pub event_id: CourseId,
}

impl Fetchable for SideBarEvent {
    type Request = SideBarEventRequest;

    const METHOD_NAME: &'static str = "GetSideBarEvent";
}
