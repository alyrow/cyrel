use serde::{Deserialize, Serialize};

use crate::celcat::entity::entity_type::{EntityTypeTrait, WrapEntityType};
use crate::celcat::entity::{entity_type, UnknownId};
use crate::celcat::resource::resource_type;

use super::calendar::CourseId;
use super::Fetchable;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SideBarEvent {
    pub federation_id: UnknownId,
    pub entity_type: WrapEntityType<entity_type::Unknown>,
    pub elements: Vec<SideBarEventElement>,
}

#[derive(Debug, Deserialize, PartialEq)]
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

#[derive(Debug, Deserialize, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::celcat::resource::RoomId;
    use serde_json::{from_str, from_value, json};

    #[test]
    fn deserialize_side_bar_event_element() {
        assert_eq!(
            from_value::<SideBarEventElement>(json!({
                "label": "Time",
                "content": "9/6/2021 8:30 AM-11:45 AM",
                "federationId": null,
                "entityType": 0,
                "assignmentContext": null,
                "containsHyperlinks": false,
                "isNotes": false,
                "isStudentSpecific": false
            }))
            .unwrap(),
            SideBarEventElement::Time(RawSideBarEventElement {
                content: Some("9/6/2021 8:30 AM-11:45 AM".to_owned()),
                federation_id: UnknownId,
                entity_type: WrapEntityType(entity_type::Unknown),
                assignment_context: None,
                contains_hyperlinks: false,
                is_notes: false,
                is_student_specific: false,
            })
        );
        assert_eq!(
            from_value::<SideBarEventElement>(json!({
                "label": "Salle",
                "content": "CHE2 Larousse haut AMPHITHÉÂTRE 673p",
                "federationId": "1042721",
                "entityType": 102,
                "assignmentContext": "a-start-end",
                "containsHyperlinks": false,
                "isNotes": false,
                "isStudentSpecific": false
            }))
            .unwrap(),
            SideBarEventElement::Room(RawSideBarEventElement {
                content: Some("CHE2 Larousse haut AMPHITHÉÂTRE 673p".to_owned()),
                federation_id: RoomId("1042721".to_owned()),
                entity_type: WrapEntityType(resource_type::Room),
                assignment_context: Some("a-start-end".to_owned()),
                contains_hyperlinks: false,
                is_notes: false,
                is_student_specific: false,
            })
        );
    }

    #[test]
    fn deserialize_side_bar_event() {
        from_value::<SideBarEvent>(json!({
            "federationId": null,
            "entityType": 0,
            "elements": []
        }))
        .unwrap();

        use std::ffi::OsStr;
        use std::fs;

        for entry in fs::read_dir("tests/resources/side_bar_event").unwrap() {
            let path = entry.unwrap().path();
            if !path.is_file() || path.extension() != Some(OsStr::new("json")) {
                continue;
            }

            let data = fs::read_to_string(&path).unwrap();
            from_str::<SideBarEvent>(&data)
                .unwrap_or_else(|e| panic!("{}: {}", path.to_str().unwrap().to_string(), e));
        }
    }
}
