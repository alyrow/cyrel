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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::ResourceType as E;

    /// Wrapper around a [`ResourceType`]
    #[derive(Debug)]
    pub struct WrapResourceType<T: ResourceType>(T);

    pub trait ResourceType {
        type Id: super::ResourceId;
        const N: E;
    }

    pub struct Formation;
    impl ResourceType for Formation {
        type Id = super::FormationId;
        const N: E = E::Formation;
    }

    pub struct Teacher;
    impl ResourceType for Teacher {
        type Id = super::TeacherId;
        const N: E = E::Teacher;
    }

    pub struct Room;
    impl ResourceType for Room {
        type Id = super::RoomId;
        const N: E = E::Room;
    }

    pub struct Group;
    impl ResourceType for Group {
        type Id = super::GroupId;
        const N: E = E::Group;
    }

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
        fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            todo!()
        }
    }
}
