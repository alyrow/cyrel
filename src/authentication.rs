use jsonrpc_core::Metadata;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::models::User;
use chrono::Utc;
use pbkdf2::password_hash::{PasswordHash, PasswordHasher, Salt};
use pbkdf2::Pbkdf2;
use std::collections::HashMap;
use std::iter::Map;
use std::time::SystemTime;
use uuid::Uuid;

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

    pub fn from_user(_user: &User) -> Self {
        todo!()
    }

    pub fn to_jwt(&self, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
        jsonwebtoken::encode(
            &Header::default(),
            self,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
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

pub struct Register {
    pub tokens: HashMap<String, User>,
}

impl Register {
    pub fn put_user(&mut self, hash: String, user: User) -> () {
        self.tokens.insert(hash, user);
        ()
    }
}
