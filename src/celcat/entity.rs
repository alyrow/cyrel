use num_traits::FromPrimitive;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use super::resource::{ResourceId, ResourceType};

#[derive(PartialEq, Debug)]
pub enum EntityType {
    Unknown,
    Resource(ResourceType),
}

impl Serialize for EntityType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            EntityType::Unknown => Serialize::serialize(&0u8, serializer),
            EntityType::Resource(rt) => Serialize::serialize(rt, serializer),
        }
    }
}

impl<'de> Deserialize<'de> for EntityType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let n = u8::deserialize(deserializer)?;
        match ResourceType::from_u8(n) {
            Some(rt) => Ok(EntityType::Resource(rt)),
            None if n == 0 => Ok(EntityType::Unknown),
            _ => Err(de::Error::invalid_value(
                de::Unexpected::Unsigned(n as u64),
                &"0, 100, 101, 102, 103 or 104",
            )),
        }
    }
}

pub trait EntityId: Serialize + for<'de> Deserialize<'de> {}

impl<T> EntityId for T where T: ResourceId {}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(from = "()", into = "()")]
pub struct UnknownId;
impl EntityId for UnknownId {}

impl From<()> for UnknownId {
    fn from(_: ()) -> Self {
        UnknownId
    }
}

impl From<UnknownId> for () {
    fn from(_: UnknownId) -> Self {}
}

pub mod entity_type {
    use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

    use super::EntityType as E;
    use crate::celcat::resource::resource_type::ResourceTypeTrait;

    #[derive(Debug)]
    pub struct WrapEntityType<T: EntityTypeTrait>(T);

    pub trait EntityTypeTrait: Default {
        type Id: super::EntityId;
        const N: E;
    }

    impl<T> EntityTypeTrait for T
    where
        T: ResourceTypeTrait,
    {
        type Id = T::Id;
        const N: E = E::Resource(T::N);
    }

    #[derive(Debug, Default)]
    pub struct Unknown;
    impl EntityTypeTrait for Unknown {
        type Id = super::UnknownId;
        const N: E = E::Unknown;
    }

    impl<T> Serialize for WrapEntityType<T>
    where
        T: EntityTypeTrait,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Serialize::serialize(&T::N, serializer)
        }
    }

    impl<'de, T> Deserialize<'de> for WrapEntityType<T>
    where
        T: EntityTypeTrait,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            fn to_int(e: E) -> u64 {
                match e {
                    E::Unknown => 0,
                    E::Resource(r) => r as u64,
                }
            }
            let n = E::deserialize(deserializer)?;
            if n == T::N {
                Ok(Self(Default::default()))
            } else {
                Err(de::Error::invalid_value(
                    de::Unexpected::Unsigned(to_int(n)),
                    &&*(to_int(T::N)).to_string(),
                ))
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::celcat::resource::resource_type;
        use serde_json::{from_value, json, to_value};

        #[test]
        fn serialize_entity_type() {
            assert_eq!(to_value(WrapEntityType(Unknown)).unwrap(), json!(0));
            assert_eq!(
                to_value(WrapEntityType(resource_type::Room)).unwrap(),
                json!(102)
            );
        }

        #[test]
        fn deserialize_entity_type() {
            from_value::<WrapEntityType<Unknown>>(json!(0)).unwrap();
            from_value::<WrapEntityType<resource_type::Group>>(json!(103)).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_value, json, to_value};

    #[test]
    fn serialize_entity_type() {
        assert_eq!(to_value(EntityType::Unknown).unwrap(), json!(0));
        assert_eq!(
            to_value(EntityType::Resource(ResourceType::Student)).unwrap(),
            json!(104)
        );
    }

    #[test]
    fn deserialize_entity_type() {
        assert_eq!(
            from_value::<EntityType>(json!(0)).unwrap(),
            EntityType::Unknown
        );
        assert_eq!(
            from_value::<EntityType>(json!(101)).unwrap(),
            EntityType::Resource(ResourceType::Teacher)
        );
    }

    #[test]
    fn serialize_unknown_id() {
        assert_eq!(to_value(UnknownId).unwrap(), json!(null));
    }

    #[test]
    fn deserialize_unknown_id() {
        from_value::<UnknownId>(json!(null)).unwrap();
        assert!(from_value::<UnknownId>(json!(0)).is_err());
    }
}
