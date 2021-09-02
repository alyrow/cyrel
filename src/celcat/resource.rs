use std::num::ParseIntError;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use void::Void;

/// Runtime representation of the type of a resource
///
/// [`resource_type::ResourceType`] for a compile-time representation.
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum ResourceType {
    Formation = 100,
    Teacher = 101,
    Room = 102,
    Group = 103,
    Student = 104,
}

pub trait ResourceId: FromStr + Serialize + for<'de> Deserialize<'de> {}

/// ID of a formation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(transparent)]
pub struct FormationId(pub String);
impl ResourceId for FormationId {}

impl FromStr for FormationId {
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_owned()))
    }
}

/// ID of a teacher
///
/// We ignore IDs that are not numbers (like `"Vac Tempo ST 27"`).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[repr(transparent)]
pub struct TeacherId(pub u64);
impl ResourceId for TeacherId {}

impl FromStr for TeacherId {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(u64::from_str(s)?))
    }
}

/// ID of a room
///
/// We ignore IDs that are not numbers.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[repr(transparent)]
pub struct RoomId(pub u64);
impl ResourceId for RoomId {}

impl FromStr for RoomId {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(u64::from_str(s)?))
    }
}

/// ID of a group
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(transparent)]
pub struct GroupId(pub String);
impl ResourceId for GroupId {}

impl FromStr for GroupId {
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_owned()))
    }
}

/// ID of a student
///
/// It corresponds to the number on the student id card.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[repr(transparent)]
pub struct StudentId(pub u64);
impl ResourceId for StudentId {}

impl FromStr for StudentId {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(u64::from_str(s)?))
    }
}

// TODO: automate it
pub mod resource_type {
    use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

    use super::ResourceType as E;

    /// Wrapper around a [`ResourceType`]
    #[derive(Debug)]
    pub struct WrapResourceType<T: ResourceType>(T);

    pub trait ResourceType: Default {
        type Id: super::ResourceId;
        const N: E;
    }

    #[derive(Default)]
    pub struct Formation;
    impl ResourceType for Formation {
        type Id = super::FormationId;
        const N: E = E::Formation;
    }

    #[derive(Default)]
    pub struct Teacher;
    impl ResourceType for Teacher {
        type Id = super::TeacherId;
        const N: E = E::Teacher;
    }

    #[derive(Default)]
    pub struct Room;
    impl ResourceType for Room {
        type Id = super::RoomId;
        const N: E = E::Room;
    }

    #[derive(Default)]
    pub struct Group;
    impl ResourceType for Group {
        type Id = super::GroupId;
        const N: E = E::Group;
    }

    #[derive(Default)]
    pub struct Student;
    impl ResourceType for Student {
        type Id = super::StudentId;
        const N: E = E::Student;
    }

    impl<T> Serialize for WrapResourceType<T>
    where
        T: ResourceType,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Serialize::serialize(&T::N, serializer)
        }
    }

    impl<'de, T> Deserialize<'de> for WrapResourceType<T>
    where
        T: ResourceType,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let n = E::deserialize(deserializer)?;
            if n == T::N {
                Ok(Self(Default::default()))
            } else {
                Err(de::Error::invalid_value(
                    de::Unexpected::Unsigned(n as u64),
                    &&*(T::N as u8).to_string(),
                ))
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use serde_json::{from_value, json, to_value};

        #[test]
        fn serialize_resource_type() {
            assert_eq!(to_value(WrapResourceType(Room)).unwrap(), json!(102));
        }

        #[test]
        fn deserialize_resource_type() {
            assert!(from_value::<WrapResourceType<Group>>(json!(103)).is_ok());
            assert!(from_value::<WrapResourceType<Student>>(json!(102)).is_err());
            assert!(from_value::<WrapResourceType<Formation>>(json!("bar")).is_err());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_value, json, to_value};

    #[test]
    fn serialize_resource_type() {
        assert_eq!(to_value(ResourceType::Formation).unwrap(), json!(100));
        assert_eq!(to_value(ResourceType::Student).unwrap(), json!(104));
    }

    #[test]
    fn deserialize_resource_type() {
        assert_eq!(
            from_value::<ResourceType>(json!(101)).unwrap(),
            ResourceType::Teacher
        );
        assert_eq!(
            from_value::<ResourceType>(json!(103)).unwrap(),
            ResourceType::Group
        );
        assert!(from_value::<ResourceType>(json!("Room")).is_err());
        assert!(from_value::<ResourceType>(json!(null)).is_err());
    }

    #[test]
    fn serialize_formation_id() {
        assert_eq!(
            to_value(FormationId("DIHB3PRF".to_owned())).unwrap(),
            json!("DIHB3PRF")
        );
    }

    #[test]
    fn deserialize_formation_id() {
        assert_eq!(
            from_value::<FormationId>(json!("2FSA31BU")).unwrap(),
            FormationId("2FSA31BU".to_owned())
        );
        assert!(from_value::<FormationId>(json!(["foo", "bar"])).is_err());
    }

    #[test]
    fn serialize_teacher_id() {
        assert_eq!(to_value(TeacherId(92144)).unwrap(), json!(92144));
    }

    #[test]
    fn deserialize_teacher_id() {
        assert_eq!(
            from_value::<TeacherId>(json!(92624)).unwrap(),
            TeacherId(92624)
        );
        assert!(from_value::<TeacherId>(json!("91402")).is_err());
    }
}
