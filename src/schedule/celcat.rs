use anyhow::anyhow;
use chrono::NaiveDateTime;
use lazy_static::lazy_static;
use log::{debug, info, trace};
use regex::Regex;
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::schedule;
use crate::SETTINGS;

#[derive(Debug)]
pub struct Celcat {
    client: reqwest::Client,
    token: String,
    logged_in: bool,
}

impl Celcat {
    pub async fn new() -> anyhow::Result<Self> {
        let client = reqwest::Client::builder().cookie_store(true).build()?;
        let token = Self::fetch_token(&client).await?;
        Ok(Self {
            client,
            token,
            logged_in: false,
        })
    }

    async fn fetch_token(client: &reqwest::Client) -> anyhow::Result<String> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r#"<input name="__RequestVerificationToken".*?value="([^"]+)""#)
                    .unwrap();
        }
        info!("fetching celcat token");
        let body = client
            .get("https://services-web.u-cergy.fr/calendar/LdapLogin")
            .send()
            .await?
            .text()
            .await?;

        if let Some(token) = RE.captures(&body).and_then(|caps| caps.get(1)) {
            Ok(token.as_str().to_owned())
        } else {
            Err(anyhow!("couldn't get the token from celcat"))
        }
    }

    pub async fn login(&mut self) -> reqwest::Result<()> {
        #[derive(Debug, Serialize)]
        struct Form<'a> {
            #[serde(rename = "Name")]
            username: &'a str,
            #[serde(rename = "Password")]
            password: &'a str,
            #[serde(rename = "__RequestVerificationToken")]
            token: &'a str,
        }

        info!("fetching celcat federation ids");
        let form = Form {
            username: &SETTINGS.celcat.username,
            password: &SETTINGS.celcat.password,
            token: &self.token,
        };
        debug!("{:?}", form);
        self.client
            .post("https://services-web.u-cergy.fr/calendar/LdapLogin/Logon")
            .form(&form)
            .send()
            .await?;

        self.logged_in = true;

        Ok(())
    }

    pub async fn fetch<F>(&mut self, req: F::Request) -> reqwest::Result<F>
    where
        F: Fetchable,
    {
        self.client
            .post(&format!(
                /* TODO: make that at compile time */
                "https://services-web.u-cergy.fr/calendar/Home/{}",
                F::METHOD_NAME,
            ))
            .form(&req)
            .send()
            .await?
            .json()
            .await
    }
}

pub trait Fetchable: for<'de> Deserialize<'de> {
    type Request: Serialize;

    const METHOD_NAME: &'static str;
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Course {
    id: String,
    start: NaiveDateTime,
    end: NaiveDateTime,
    all_day: bool,
    description: String,
    background_color: String,
    text_color: String,
    departement: Option<String>,
    faculty: Option<String>,
    event_category: Option<String>,
    sites: Option<Vec<String>>,
    modules: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub enum EntityType {
    Unknown,
    Resource(ResourceType),
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum ResourceType {
    Formation = 100,
    Professor = 101,
    Room = 102,
    Group = 103,
    Student = 104,
}

// TODO: find a crate that automates it
mod resource_type {
    use super::ResourceType as E;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Debug)]
    pub struct WrapResourceType<T: ResourceType>(T);

    pub trait ResourceType {
        const N: E;
    }

    pub struct Formation;
    impl ResourceType for Formation {
        const N: E = E::Formation;
    }
    pub struct Professor;
    impl ResourceType for Professor {
        const N: E = E::Professor;
    }
    pub struct Room;
    impl ResourceType for Room {
        const N: E = E::Room;
    }
    pub struct Group;
    impl ResourceType for Group {
        const N: E = E::Group;
    }
    pub struct Student;
    impl ResourceType for Student {
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
            Deserialize::deserialize(deserializer)
        }
    }
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
pub struct CalendarDataRequest {
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
    pub res_type: ResourceType,
    pub cal_view: CalView,
    pub federation_ids: String, // TODO: check possible values
    pub colour_scheme: i64,
}

impl Fetchable for Vec<Course> {
    type Request = CalendarDataRequest;

    const METHOD_NAME: &'static str = "GetCalendarData";
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SideBarEvent {
    pub federation_id: Option<String>,
    pub entity_type: EntityType,
    pub elements: Vec<SideBarEventElement>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SideBarEventElement {
    pub label: SideBarEventElementLabel,
    pub content: Option<String>,
    pub federation_id: Option<String>,
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

use resource_type::ResourceType as __ResourceType;

pub trait Resource: Sized {
    type ResourceType: __ResourceType;

    const RESOURCE_TYPE: ResourceType = Self::ResourceType::N;

    fn from_raw(raw: RawResource) -> anyhow::Result<Self>;
}

fn deserialize_resources<'de, D, R>(deserializer: D) -> Result<Vec<R>, D::Error>
where
    R: Resource,
    D: Deserializer<'de>,
{
    <Vec<RawResource> as Deserialize>::deserialize(deserializer)?
        .into_iter()
        .map(|r| R::from_raw(r))
        .collect::<anyhow::Result<Vec<R>>>()
        .map_err(|e| D::Error::custom(e.to_string()))
}

#[derive(Debug, Deserialize)]
pub struct RawResource {
    pub id: String,
    pub text: String,
    pub dept: String,
}

#[derive(Debug)]
pub struct Formation {
    pub id: String,
}

impl Resource for Formation {
    type ResourceType = resource_type::Formation;

    fn from_raw(raw: RawResource) -> anyhow::Result<Self> {
        Ok(Self { id: raw.id })
    }
}
