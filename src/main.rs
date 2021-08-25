#[macro_use]
extern crate diesel;

mod models;
mod schema;

use std::boxed::Box;
use std::env;
use std::sync::{Arc, Mutex};

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use jsonrpc_core::*;
use jsonrpc_derive::rpc;
use jsonrpc_http_server::*;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use pbkdf2::password_hash::{PasswordHash, PasswordVerifier};
use pbkdf2::Pbkdf2;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::User;
use crate::schema::users;

#[derive(Debug, Default, Clone)]
struct Meta {
    jwt: Option<String>,
}

impl Metadata for Meta {}

#[rpc(server)]
pub trait Rpc {
    type Metadata;

    #[rpc(name = "ping")]
    fn ping(&self) -> Result<String>;

    #[rpc(name = "login")]
    fn login(&self, username: String, password: String) -> Result<String>;
}

struct RpcImpl<'a> {
    db: Arc<Mutex<SqliteConnection>>,
    jwt_secret: &'a str,
    rng: StdRng,
}

impl Rpc for RpcImpl<'static> {
    type Metadata = Meta;

    fn ping(&self) -> Result<String> {
        Ok("pong".to_owned())
    }

    fn login(&self, username: String, password: String) -> Result<String> {
        let user: User = {
            let db = self.db.lock().unwrap();
            users::dsl::users
                .filter(users::dsl::username.eq(username))
                .first(&*db)
                .unwrap()
        };
        let hash = PasswordHash::new(&user.password).unwrap();
        if Pbkdf2.verify_password(password.as_bytes(), &hash).is_ok() {
            let jwt = Claims::from_user(&user).to_jwt(self.jwt_secret).unwrap();
            Ok(jwt)
        } else {
            todo!()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

impl Claims {
    fn from_meta(
        meta: &Meta,
        secret: &str,
    ) -> std::result::Result<Option<Self>, jsonwebtoken::errors::Error> {
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

    fn from_user(user: &User) -> Self {
        todo!()
    }

    fn to_jwt(&self, secret: &str) -> std::result::Result<String, jsonwebtoken::errors::Error> {
        jsonwebtoken::encode(
            &Header::default(),
            self,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
    }
}

fn main() {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_conn =
        SqliteConnection::establish(&db_url).expect(&format!("Error connecting to {}", db_url));

    // FIXME: make it random (more difficult for debugging)
    let jwt_secret = Box::leak(
        env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set")
            .into_boxed_str(),
    );

    let mut io = MetaIoHandler::default();
    let rpc = RpcImpl {
        db: Arc::new(Mutex::new(db_conn)),
        jwt_secret,
        rng: StdRng::from_entropy(),
    };

    io.extend_with(rpc.to_delegate());

    let server = ServerBuilder::new(io)
        .cors_allow_headers(cors::AccessControlAllowHeaders::Any)
        .meta_extractor(|req: &hyper::Request<hyper::Body>| {
            let jwt = req
                .headers()
                .get(hyper::header::AUTHORIZATION)
                .map(|h| h.to_str().ok())
                .flatten()
                .map(|s| s.strip_prefix("Bearer ")) // FIXME: reliable?
                .flatten()
                .map(|s| s.to_owned());
            Meta { jwt }
        })
        .start_http(&"127.0.0.1:3030".parse().unwrap())
        .unwrap();

    server.wait();
}
