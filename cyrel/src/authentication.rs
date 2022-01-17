use std::collections::HashMap;

use jsonrpc_core::Metadata;
use jsonrpc_http_server::Rpc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use pbkdf2::password_hash::{PasswordHasher, Salt};
use pbkdf2::Pbkdf2;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{error, info, warn};

use crate::db::Db;
use crate::models::User;
use crate::rpc::{RpcError, RpcImpl};
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

pub struct CheckUser {}

impl CheckUser {
    pub async fn jwt_check(pool: &PgPool, sub: String) -> jsonrpc_core::Result<User> {
        let user: User = {
            let result = Db::match_user_by_id(&pool, sub.parse::<i64>().unwrap()).await;

            match result {
                Ok(user) => user,
                Err(_) => {
                    warn!("unknown connected user {}", sub);
                    return Err(RpcError::IncorrectLoginInfo.into());
                }
            }
        };
        Ok(user)
    }

    pub async fn logged_user_get(pool: &PgPool, meta: Meta) -> Option<User> {
        let user = {
            let claims = match Claims::from_meta(&meta, &SETTINGS.jwt.secret) {
                Ok(claims) => claims,
                Err(err) => {
                    warn!("{}", err.to_string());
                    return None;
                }
            };
            let claims = match claims {
                Some(claims) => claims,
                None => {
                    warn!("User not logged!");
                    return None;
                }
            };
            let sub = claims.sub.to_owned();
            let check_result = CheckUser::jwt_check(RpcImpl::get_postgres(), sub).await;
            match check_result {
                Ok(user) => user,
                Err(err) => {
                    warn!("{}", err.to_string());
                    return None;
                }
            }
        };
        Some(user)
    }
}

#[derive(std::fmt::Debug)]
pub struct Register {
    pub tokens: HashMap<String, User>,
}

impl Register {
    pub fn put_user(&mut self, hash: String, user: User) -> () {
        self.tokens.insert(hash, user);
        ()
    }
}

pub struct HashFunction {}

impl HashFunction {
    pub fn hash_password(password: String, salt: String) -> String {
        Pbkdf2
            .hash_password_simple(password.as_bytes(), &Salt::new(&*salt).unwrap())
            .unwrap()
            .to_string()
    }
}
