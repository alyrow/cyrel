use jsonrpc_core::Metadata;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use pbkdf2::{
    password_hash::{PasswordHasher, Salt},
    Pbkdf2,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::warn;

use crate::models::User;
use crate::SETTINGS;

#[derive(Debug, Default, Clone)]
pub struct Meta {
    pub jwt: Option<String>,
}

impl Metadata for Meta {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

impl Claims {
    pub fn from_meta(
        meta: &Meta,
        secret: &str,
    ) -> Result<Option<Self>, jsonwebtoken::errors::Error> {
        let jwt = match &meta.jwt {
            Some(s) => s,
            None => {
                return Ok(None);
            }
        };
        jsonwebtoken::decode(
            jwt,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map(|j| Some(j.claims))
    }

    pub fn from_user(user: &User) -> Self {
        let time = chrono::offset::Utc::now() + chrono::Duration::days(14);
        Claims {
            sub: user.id.to_string(),
            exp: time.timestamp() as usize,
        }
    }

    pub fn to_jwt(&self, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
        jsonwebtoken::encode(
            &Header::default(),
            self,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
    }
}

pub async fn logged_user_get(pool: &PgPool, meta: Meta) -> Option<User> {
    let claims = match Claims::from_meta(&meta, &SETTINGS.jwt.secret) {
        Ok(Some(claims)) => claims,
        Ok(None) => {
            warn!("User not logged!");
            return None;
        },
        Err(err) => {
            warn!("{}", err.to_string());
            return None;
        },
    };

    match sqlx::query_as!(
        User,
        "select * from users where id = $1",
        claims.sub.parse::<i64>().unwrap(),
    ).fetch_one(pool).await {
        Ok(user) => Some(user),
        Err(err) => {
            warn!("{}", err.to_string());
            None
        }
    }
}

pub fn hash_password(password: String, salt: String) -> String {
    Pbkdf2
        .hash_password_simple(password.as_bytes(), &Salt::new(&*salt).unwrap())
        .unwrap()
        .to_string()
}
