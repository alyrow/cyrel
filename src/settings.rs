use clap::ArgMatches;
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Jwt {
    pub secret: String,
}

#[derive(Debug, Deserialize)]
pub struct Database {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub jwt: Jwt,
    pub database: Database,
    pub port: u16,
}

impl Settings {
    pub fn new(matches: &ArgMatches) -> Result<Self, ConfigError> {
        let mut s = Config::default();

        if let Some(f) = matches.value_of("CONFIG") {
            s.merge(File::with_name(f))?;
        }
        s.merge(Environment::new().separator("_").ignore_empty(true))?;

        if let Some(p) = matches.value_of("PORT") {
            s.set("port", p)?;
        }

        s.try_into()
    }
}
