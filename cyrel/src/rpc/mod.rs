mod error;

use crate::db::match_user;
use chrono::NaiveDateTime;
use jsonrpc_core::BoxFuture;
use jsonrpc_derive::rpc;
use log::{error, info, warn};
use once_cell::sync::OnceCell;
use pbkdf2::password_hash::{PasswordHash, PasswordVerifier};
use pbkdf2::Pbkdf2;
use rand::prelude::StdRng;
use sqlx::PgPool;

use crate::authentication::{Claims, Meta};
use crate::models::User;
use crate::schedule::celcat::fetch_calendar;
use crate::schedule::Course;
use crate::SETTINGS;

pub use self::error::RpcError;
pub use self::rpc_impl_Rpc::gen_server;

#[rpc(server)]
pub trait Rpc {
    type Metadata;

    #[rpc(name = "ping")]
    fn ping(&self) -> jsonrpc_core::Result<String>;

    #[rpc(name = "login", params = "named")]
    fn login(&self, id: i64, password: String) -> BoxFuture<jsonrpc_core::Result<String>>;

    #[rpc(meta, name = "schedule_get", params = "named")]
    fn schedule_get(
        &self,
        meta: Self::Metadata,
        start: NaiveDateTime,
        end: NaiveDateTime,
        group: i64,
    ) -> BoxFuture<jsonrpc_core::Result<Vec<Course>>>;
}

pub struct RpcImpl {
    pub rng: StdRng,
}

static POSTGRES: OnceCell<PgPool> = OnceCell::new();

impl RpcImpl {
    pub async fn new(url: &String, rng: StdRng) -> RpcImpl {
        RpcImpl::create_pg_pool(url).await;
        return RpcImpl { rng };
    }

    pub async fn create_pg_pool(database_url: &String) {
        let pool = PgPool::connect(database_url)
            .await
            .expect("Failed to create pool.");
        POSTGRES
            .set(pool)
            .expect("Postgresql global pool must set success")
    }

    #[inline]
    pub fn get_postgres() -> &'static PgPool {
        unsafe { POSTGRES.get_unchecked() }
    }
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

    fn login(&self, id: i64, password: String) -> BoxFuture<jsonrpc_core::Result<String>> {
        Box::pin(async move {
            let db = RpcImpl::get_postgres();

            let user: User = {
                let result = match_user(&db, id).await;

                match result {
                    Ok(user) => user,
                    Err(_) => {
                        warn!("{} isn't a know id", id);
                        return Err(RpcError::IncorrectLoginInfo.into());
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
                info!("{} logged in", id);
                Ok(jwt)
            } else {
                warn!("{} failed to log in", id);
                Err(RpcError::IncorrectLoginInfo.into())
            }
        })
    }

    fn schedule_get(
        &self,
        _meta: Self::Metadata,
        start: NaiveDateTime,
        end: NaiveDateTime,
        group: i64,
    ) -> BoxFuture<jsonrpc_core::Result<Vec<Course>>> {
        Box::pin(async move {
            fetch_calendar(start, end, group)
                .await
                .map_err(|err| jsonrpc_core::Error {
                    code: jsonrpc_core::ErrorCode::ServerError(-32000),
                    message: format!("{}", err),
                    data: None,
                })
        })
    }
}
