use anyhow::anyhow;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use log::{debug, info};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::calendar;
use crate::SETTINGS;

#[derive(Debug)]
pub struct Celcat {
    client: reqwest::Client,
    token: String,
    logged_in: bool,
}

impl Celcat {
    pub async fn new() -> anyhow::Result<Self> {
        let client = reqwest::Client::new();
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

    pub async fn fetch(&mut self, req: Request) -> reqwest::Result<Course> {
        self.client
            .post("https://services-web.u-cergy.fr/calendar/Home/GetCalendarData")
            .form(&req)
            .send()
            .await?
            .json()
            .await
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Course {
    id: String,
    start: DateTime<Utc>, // FIXME: UTC or local?
    end: DateTime<Utc>,
    all_day: bool,
    description: String,
    background_color: String,
    text_color: String,
    departement: String,
    faculty: String,
    event_category: String,
    sites: Vec<String>,
    modules: Vec<String>,
}

impl From<Course> for calendar::Course {
    fn from(c: Course) -> Self {
        Self {
            uid: c.id,
            start: c.start,
            end: c.end,
            category: todo!(),
            name: todo!(),
            location: todo!(),
            prof: todo!(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CalView {
    Month,
    // TODO: extend this
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub res_type: i64, // TODO: check possible values
    pub cal_view: CalView,
    pub federation_ids: String, // TODO: check possible value
}
