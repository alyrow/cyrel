use serde::{Deserialize, Serialize};

use crate::celcat::EntityType;

use super::Fetchable;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SideBarEvent {
    pub federation_id: Option<String>, // TODO
    pub entity_type: EntityType,
    pub elements: Vec<SideBarEventElement>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SideBarEventElement {
    pub label: SideBarEventElementLabel,
    pub content: Option<String>,
    pub federation_id: Option<String>, // TODO
    pub entity_type: EntityType,
    pub assignment_context: Option<String>,
    pub contains_hyperlinks: bool,
    pub is_notes: bool,
    pub is_student_specific: bool,
}

#[derive(Debug, Deserialize)]
pub enum SideBarEventElementLabel {
    Time,
    #[serde(rename = "Catégorie")]
    Category,
    #[serde(rename = "Matière")]
    Subject,
    #[serde(rename = "Salle")]
    Room,
    #[serde(rename = "Enseignant")]
    Teacher,
    #[serde(rename = "Notes")]
    Grades,
    Name,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SideBarEventRequest {
    pub event_id: String,
}

impl Fetchable for SideBarEvent {
    type Request = SideBarEventRequest;

    const METHOD_NAME: &'static str = "GetSideBarEvent";
}
