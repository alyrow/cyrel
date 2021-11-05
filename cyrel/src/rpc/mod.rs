use std::collections::HashMap;

use chrono::NaiveDateTime;
use jsonrpc_core::BoxFuture;
use jsonrpc_derive::rpc;
use jsonwebtoken::errors::Error;
use log::{error, info, warn};
use once_cell::sync::OnceCell;
use pbkdf2::password_hash::{PasswordHash, PasswordVerifier};
use pbkdf2::Pbkdf2;
use rand::prelude::StdRng;
use sqlx::PgPool;

use crate::authentication::{CheckUser, Claims, HashFunction, Meta, Register};
use crate::db::Db;
use crate::email::Email;
use crate::models::{Department, Identity, User};
use crate::schedule::celcat::fetch_calendar;
use crate::schedule::Course;
use crate::SETTINGS;

pub use self::error::RpcError;
pub use self::rpc_impl_Rpc::gen_server;

mod error;

#[rpc(server)]
pub trait Rpc {
    type Metadata;

    #[rpc(name = "ping")]
    fn ping(&self) -> jsonrpc_core::Result<String>;

    #[rpc(name = "login", params = "named")]
    fn login(&self, email: String, password: String) -> BoxFuture<jsonrpc_core::Result<String>>;

    #[rpc(name = "register_1", params = "named")]
    fn register_1(
        &self,
        ldap: i64,
        department: String,
        email: String,
    ) -> BoxFuture<jsonrpc_core::Result<String>>;

    #[rpc(name = "register_2", params = "named")]
    fn register_2(&self, hash: String) -> BoxFuture<jsonrpc_core::Result<Identity>>;

    #[rpc(name = "register_3", params = "named")]
    fn register_3(
        &self,
        hash: String,
        firstname: String,
        lastname: String,
        password: String,
    ) -> BoxFuture<jsonrpc_core::Result<String>>;

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
static mut TOKENS: OnceCell<Register> = OnceCell::new();

impl RpcImpl {
    pub async fn new(url: &String, rng: StdRng) -> RpcImpl {
        RpcImpl::create_pg_pool(url).await;
        RpcImpl::create_tokens().await;
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

    pub async fn create_tokens() {
        unsafe {
            TOKENS
                .set(Register {
                    tokens: HashMap::<String, User>::new(),
                })
                .expect("TOKENS global mut must set success")
        }
    }

    #[inline]
    pub fn get_tokens() -> &'static mut Register {
        unsafe { TOKENS.get_mut().expect("Blblbl") }
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

    fn login(&self, email: String, password: String) -> BoxFuture<jsonrpc_core::Result<String>> {
        Box::pin(async move {
            let pool = RpcImpl::get_postgres();

            let user: User = {
                let result = Db::match_user_by_email(&pool, email.to_owned()).await;

                match result {
                    Ok(user) => user,
                    Err(_) => {
                        warn!("{} isn't a know email", email);
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
                info!("{} logged in", user.id);
                Ok(jwt)
            } else {
                warn!("{} failed to log in", user.id);
                Err(RpcError::IncorrectLoginInfo.into())
            }
        })
    }

    fn register_1(
        &self,
        ldap: i64,
        department: String,
        email: String,
    ) -> BoxFuture<jsonrpc_core::Result<String>> {
        Box::pin(async move {
            let pool = RpcImpl::get_postgres();

            let dpmt: Department = {
                let result = Db::match_department(&pool, department.clone()).await;

                match result {
                    Ok(dpmt) => dpmt,
                    Err(_) => {
                        warn!("department {} is unknown", department);
                        return Err(RpcError::UnknownDepartment.into());
                    }
                }
            };

            let mut email = email;
            email.push_str("@");
            email.push_str(&*dpmt.domain);

            let mut user: User = {
                let result = Db::match_user_by_id(&pool, ldap).await;

                match result {
                    Ok(_) => {
                        warn!("user {} is already registered", ldap);
                        return Err(RpcError::AlreadyRegistered.into());
                    }
                    Err(_) => User {
                        id: ldap,
                        firstname: "".to_string(),
                        lastname: "".to_string(),
                        email: email.to_owned(),
                        password: "".to_string(),
                    },
                }
            };

            let result = Db::match_user_by_email(&pool, email.to_owned()).await;
            match result {
                Ok(u) => {
                    warn!("email {} is already used for user {}", email, u.id);
                    return Err(RpcError::AlreadyRegistered.into());
                }
                Err(_) => {}
            }

            let validity: (String, String) = {
                let result = Db::match_celcat_student(&pool, ldap, department.clone()).await;

                match result {
                    Ok(data) => data,
                    Err(_) => {
                        warn!("User {} in department: {} is unknown", ldap, department);
                        return Err(RpcError::IncorrectLoginInfo.into());
                    }
                }
            };

            user.firstname = validity.0;
            user.lastname = validity.1;

            let hash = uuid::Uuid::new_v4().to_string();
            info!("{}", hash);
            let mut register = RpcImpl::get_tokens();
            register.put_user(hash.to_owned(), user);
            let email_response = Email::send_verification_email(email, department, hash);
            if !email_response.is_positive() {
                warn!("{}", email_response.code().to_string());
                return Err(RpcError::UnknownError.into());
            }

            Ok("Code sent".to_string())
        })
    }

    fn register_2(&self, hash: String) -> BoxFuture<jsonrpc_core::Result<Identity>> {
        Box::pin(async move {
            let mut register = RpcImpl::get_tokens();
            if !register.tokens.contains_key(&*hash.to_owned()) {
                warn!(
                    "Someone tried to use an used or inexistant token: {}",
                    hash.to_owned()
                );
                return Err(RpcError::RegistrationTokenUsed.into());
            }
            let user_saved = register.tokens.get(&hash).unwrap();
            Ok(Identity {
                firstname: user_saved.firstname.to_owned(),
                lastname: user_saved.lastname.to_owned(),
            })
        })
    }

    fn register_3(
        &self,
        hash: String,
        firstname: String,
        lastname: String,
        password: String,
    ) -> BoxFuture<jsonrpc_core::Result<String>> {
        Box::pin(async move {
            let mut register = RpcImpl::get_tokens();
            if !register.tokens.contains_key(&*hash.to_owned()) {
                warn!(
                    "Someone tried to use an used or inexistant token: {}",
                    hash.to_owned()
                );
                return Err(RpcError::RegistrationTokenUsed.into());
            }
            let user_saved = register.tokens.get(&hash).unwrap();
            let id_str = user_saved.id.to_string();
            let pass_hasher = HashFunction::hash_password(password.to_owned(), id_str.to_owned());
            let user = User {
                id: user_saved.id.to_owned(),
                firstname,
                lastname,
                email: user_saved.email.to_owned(),
                password: pass_hasher,
            };
            let pool = RpcImpl::get_postgres();

            let validity: () = {
                let result = Db::insert_user(&pool, user).await;
                match result {
                    Ok(data) => data,
                    Err(err) => {
                        warn!("{}", err.to_string());
                        return Err(RpcError::Unimplemented.into());
                    }
                }
            };
            register.tokens.remove(&*hash);
            Ok("Account created!".to_string())
        })
    }

    fn schedule_get(
        &self,
        meta: Self::Metadata,
        start: NaiveDateTime,
        end: NaiveDateTime,
        group: i64,
    ) -> BoxFuture<jsonrpc_core::Result<Vec<Course>>> {
        Box::pin(async move {
            let user: User = {
                let claims = match Claims::from_meta(&meta, &SETTINGS.jwt.secret) {
                    Ok(claims) => claims,
                    Err(err) => {
                        warn!("{}", err.to_string());
                        return Err(RpcError::IncorrectLoginInfo.into());
                    }
                };
                let claims = claims.unwrap();
                let sub = claims.sub.to_owned();
                let check_result = CheckUser::jwt_check(RpcImpl::get_postgres(), sub).await;
                match check_result {
                    Ok(user) => user,
                    Err(err) => {
                        warn!("{}", err.to_string());
                        return Err(RpcError::IncorrectLoginInfo.into());
                    }
                }
            };
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
