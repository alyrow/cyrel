mod fetchable;
mod resource;

use anyhow::anyhow;
use lazy_static::lazy_static;
use log::{debug, info};
use num_traits::FromPrimitive;
use regex::Regex;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::SETTINGS;

use self::fetchable::Fetchable;
use self::resource::ResourceType;

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

        // TODO: We get disconnected after a while
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
}
