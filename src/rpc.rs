use std::sync::{Arc, Mutex};

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use jsonrpc_derive::rpc;
use pbkdf2::Pbkdf2;
use pbkdf2::password_hash::{PasswordHash, PasswordVerifier};
use rand::prelude::StdRng;

use crate::authentication::{Meta, Claims};
use crate::{models::User, schema::users};

pub use self::rpc_impl_Rpc::gen_server;

#[rpc(server)]
pub trait Rpc {
    type Metadata;

    #[rpc(name = "ping")]
    fn ping(&self) -> jsonrpc_core::Result<String>;

    #[rpc(name = "login")]
    fn login(&self, username: String, password: String) -> jsonrpc_core::Result<String>;
}

pub struct RpcImpl<'a> {
    pub db: Arc<Mutex<SqliteConnection>>,
    pub jwt_secret: &'a str,
    pub rng: StdRng,
}

impl Rpc for RpcImpl<'static> {
    type Metadata = Meta;

    fn ping(&self) -> jsonrpc_core::Result<String> {
        Ok("pong".to_owned())
    }

    fn login(&self, username: String, password: String) -> jsonrpc_core::Result<String> {
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
