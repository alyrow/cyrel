use std::sync::{Arc, Mutex};

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use jsonrpc_derive::rpc;
use log::{error, info, warn};
use pbkdf2::password_hash::{PasswordHash, PasswordVerifier};
use pbkdf2::Pbkdf2;
use rand::prelude::StdRng;

use crate::authentication::{Claims, Meta};
use crate::SETTINGS;
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

pub struct RpcImpl {
    pub db: Arc<Mutex<SqliteConnection>>,
    pub rng: StdRng,
}

macro_rules! server_error {
    ($e:expr) => {
        match $e {
            Ok(a) => a,
            Err(err) => {
                error!("{}", err);
                return Err(jsonrpc_core::Error {
                    // TODO: define error codes
                    code: jsonrpc_core::ErrorCode::ServerError(-32000),
                    message: format!("{}", err),
                    data: None,
                });
            }
        }
    };
}

impl Rpc for RpcImpl {
    type Metadata = Meta;

    fn ping(&self) -> jsonrpc_core::Result<String> {
        info!("pinged");
        Ok("pong".to_owned())
    }

    fn login(&self, username: String, password: String) -> jsonrpc_core::Result<String> {
        let user: User = {
            let db = server_error! {
                self.db.lock()
            };

            match users::dsl::users
                .filter(users::dsl::username.eq(&username))
                .first(&*db)
            {
                Ok(user) => user,
                Err(_) => {
                    warn!("{} isn't a know username", username);
                    return Err(jsonrpc_core::Error {
                        // TODO: define error codes
                        code: jsonrpc_core::ErrorCode::ServerError(1),
                        message: "unknown username".to_owned(),
                        data: None,
                    });
                }
            }
        };

        let hash = server_error! {
            PasswordHash::new(&user.password)
        };
        if Pbkdf2.verify_password(password.as_bytes(), &hash).is_ok() {
            let jwt = server_error! {
                Claims::from_user(&user).to_jwt(&SETTINGS.jwt.secret)
            };
            info!("{} logged in", username);
            Ok(jwt)
        } else {
            warn!("{} failed to log in", username);
            Err(jsonrpc_core::Error {
                // TODO: define error codes
                code: jsonrpc_core::ErrorCode::ServerError(2),
                message: "invalid password".to_owned(),
                data: None,
            })
        }
    }
}
