mod error;

use std::sync::{Arc, Mutex, MutexGuard};

use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use jsonrpc_core::BoxFuture;
use jsonrpc_derive::rpc;
use log::{error, info, warn};
use pbkdf2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, Salt};
use pbkdf2::{Algorithm, Params, Pbkdf2};
use rand::prelude::StdRng;

use crate::authentication::{Claims, HashFunction, Meta, Register};
use crate::schedule::celcat::{fetch_calendar, GroupId};
use crate::schedule::Course;
use crate::SETTINGS;
use crate::{models::User, schema::users};
use crate::{models::Department, schema::departments};

pub use self::error::RpcError;
pub use self::rpc_impl_Rpc::gen_server;
use crate::models::NewUser;
use crate::users::Email;
use diesel::dsl::exists;
use diesel::select;
use std::ptr::null;
use uuid::Uuid;

#[rpc(server)]
pub trait Rpc {
    type Metadata;

    #[rpc(name = "ping")]
    fn ping(&self) -> jsonrpc_core::Result<String>;

    #[rpc(name = "login", params = "named")]
    fn login(&self, username: String, password: String) -> jsonrpc_core::Result<String>;

    #[rpc(name = "register_1", params = "named")]
    fn register_1(
        &self,
        ldap: i32,
        department: String,
        email: String,
    ) -> jsonrpc_core::Result<String>;

    #[rpc(name = "register_2", params = "named")]
    fn register_2(
        &self,
        hash: String,
        username: String,
        password: String,
        groups: i32,
    ) -> jsonrpc_core::Result<String>;

    #[rpc(meta, name = "schedule_get", params = "named")]
    fn schedule_get(
        &self,
        meta: Self::Metadata,
        start: NaiveDateTime,
        end: NaiveDateTime,
        group: GroupId,
    ) -> BoxFuture<jsonrpc_core::Result<Vec<Course>>>;
}

pub struct RpcImpl {
    pub db: Arc<Mutex<SqliteConnection>>,
    pub rng: StdRng,
    pub register: Arc<Mutex<Register>>,
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

    fn register_1(
        &self,
        ldap: i32,
        department: String,
        email: String,
    ) -> jsonrpc_core::Result<String> {

        let dpmt: Department = {
            let db = server_error! {
                self.db.lock()
            };

            match departments::dsl::departments
                .filter(departments::dsl::id.eq(&department))
                .first::<Department>(&*db)
            {
                Ok(dpmt) => dpmt,
                Err(_) => {
                    warn!("department {} is unknown", department);
                    return Err(RpcError::UnknownDepartment.into());
                },
            }
        };

        let mut email = email;
        email.push_str("@");
        email.push_str(&*dpmt.domain);

        let user: User = {
            let db = server_error! {
                self.db.lock()
            };

            match users::dsl::users
                .filter(users::dsl::id.eq(&ldap))
                .first::<User>(&*db)
            {
                Ok(_) => {
                    warn!("user {} is already registered", ldap);
                    return Err(RpcError::AlreadyRegistered.into());
                }
                Err(_) => User {
                    id: ldap,
                    username: "".to_string(),
                    email: email.to_owned(),
                    password: "".to_string(),
                    groups: -1,
                },
            }
        };

        let hash = uuid::Uuid::new_v4().to_string();
        info!("{}", hash);
        let mut register = server_error! {
            self.register.lock()
        };
        register.put_user(hash.to_owned(), user);
        let email_response = Email::send_verification_email(email.to_owned(), department, hash);
        if !email_response.is_positive() {
            warn!("{}", email_response.code().to_string());
            return Err(RpcError::UnknownError.into());
        }

        Ok("Code sent".to_string())
    }

    fn register_2(
        &self,
        hash: String,
        username: String,
        password: String,
        groups: i32,
    ) -> jsonrpc_core::Result<String> {
        let mut register = server_error! {
            self.register.lock()
        };

        if !register.tokens.contains_key(&*hash.to_owned()) {
            warn!(
                "Someone tried to use an used or inexistant token: {}",
                hash.to_owned()
            );
            return Err(RpcError::RegistrationTokenUsed.into());
        }
        let mut user: User = {
            let db = server_error! {
                self.db.lock()
            };

            match users::dsl::users
                .filter(users::dsl::username.eq(&username))
                .first::<User>(&*db)
            {
                Ok(_) => {
                    warn!("{} is a used username", username);
                    return Err(RpcError::IncorrectLoginInfo.into());
                }
                Err(_) => User {
                    id: -1,
                    username: username.to_owned(),
                    email: "".to_string(),
                    password: "".to_string(),
                    groups: groups.to_owned(),
                },
            }
        };
        let user_saved = register.tokens.get(&hash).unwrap();
        user.id = user_saved.id.to_owned();
        user.email = user_saved.email.to_owned();
        let id_str = user.id.to_string();
        let pass_hasher = HashFunction::hash_password(password.to_owned(), id_str.to_owned());
        user.password = pass_hasher;
        let new_user = NewUser {
            id: user.id.to_owned(),
            username: &*user.username.to_owned(),
            email: &*user.email.to_owned(),
            password: &*user.password.to_owned(),
            groups: user.groups.to_owned(),
        };
        let db = server_error! {
            self.db.lock()
        };
        let insertion = diesel::insert_into(users::dsl::users)
            .values(new_user)
            .execute(&*db);
        if insertion.is_err() {
            warn!("{}", insertion.err().unwrap().to_string());
            return Err(RpcError::Unimplemented.into());
        }
        register.tokens.remove(&*hash);
        Ok("Account created!".to_string())
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
            info!("{} logged in", username);
            Ok(jwt)
        } else {
            warn!("{} failed to log in", username);
            Err(RpcError::IncorrectLoginInfo.into())
        }
    }

    fn schedule_get(
        &self,
        _meta: Self::Metadata,
        start: NaiveDateTime,
        end: NaiveDateTime,
        group: GroupId,
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
