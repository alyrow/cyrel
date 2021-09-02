mod fetchable;
mod resource;

use anyhow::anyhow;

use lazy_static::lazy_static;
use log::{debug, info};
use regex::Regex;
use serde::{Deserialize, Serialize};

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

#[derive(Deserialize, Serialize, PartialEq, Debug)]
// FIXME: Write implementations of Serialize and Deserialize
pub enum EntityType {
    Unknown,
    Resource(ResourceType),
}
