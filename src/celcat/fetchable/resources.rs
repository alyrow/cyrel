use std::str::FromStr;

use serde::{de, Deserialize, Deserializer, Serialize};

use crate::celcat::resource::resource_type::ResourceType as TypeResourceType;
use crate::celcat::resource::{resource_type, ModuleId, ResourceType};

use super::Fetchable;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceList<R: Resource> {
    pub total: u64,
    #[serde(deserialize_with = "deserialize_resources")]
    pub results: Vec<R>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceListRequest<T: resource_type::ResourceType> {
    pub my_resources: bool,
    pub search_term: String,
    pub page_size: u64,
    pub page_numer: u64,
    #[serde(bound(serialize = "T: resource_type::ResourceType"))]
    pub res_type: resource_type::WrapResourceType<T>,
    /// Milliseconds since Epoch
    #[serde(rename = "_")]
    pub timestamp: u128,
}

impl<R> Fetchable for ResourceList<R>
where
    R: Resource,
{
    type Request = ResourceListRequest<R::ResourceType>;

    const METHOD_NAME: &'static str = "ReadResourceListItems";
}

pub trait Resource: Sized {
    type ResourceType: TypeResourceType;

    const RESOURCE_TYPE: ResourceType = Self::ResourceType::N;

    fn from_raw(raw: RawResource) -> anyhow::Result<Self>;

    fn id(&self) -> <Self::ResourceType as TypeResourceType>::Id;
}

fn deserialize_resources<'de, D, R>(deserializer: D) -> Result<Vec<R>, D::Error>
where
    R: Resource,
    D: Deserializer<'de>,
{
    <Vec<RawResource> as Deserialize>::deserialize(deserializer)?
        .into_iter()
        .map(R::from_raw)
        .collect::<anyhow::Result<Vec<R>>>()
        .map_err(de::Error::custom)
}

#[derive(Debug, Deserialize)]
pub struct RawResource {
    pub id: String,
    pub text: String,
    pub dept: Option<String>,
}

#[derive(Debug)]
pub struct Formation {
    pub id: ModuleId,
}

impl Resource for Formation {
    type ResourceType = resource_type::Module;

    fn from_raw(raw: RawResource) -> anyhow::Result<Self> {
        Ok(Self {
            id: ModuleId::from_str(&raw.id)?,
        })
    }

    fn id(&self) -> <Self::ResourceType as TypeResourceType>::Id {
        self.id.clone()
    }
}
