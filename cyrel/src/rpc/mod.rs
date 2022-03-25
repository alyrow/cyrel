use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use chrono::{NaiveDateTime, Utc};
use jsonrpc_core::BoxFuture;
use jsonrpc_derive::rpc;
use lettre::{
    transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport,
    Tokio1Executor,
};
use pbkdf2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Pbkdf2,
};
use sqlx::PgPool;
use tracing::{debug, error, info, warn};

use crate::authentication::{self, Claims, Meta};
use crate::email;
use crate::models::{Department, Group, Identity, User};
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

    #[rpc(name = "time")]
    fn time(&self) -> jsonrpc_core::Result<NaiveDateTime>;

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

    #[rpc(meta, name = "is_logged")]
    fn is_logged(&self, meta: Self::Metadata) -> BoxFuture<jsonrpc_core::Result<bool>>;

    #[rpc(meta, name = "my_groups_get", params = "named")]
    fn my_groups_get(&self, meta: Self::Metadata) -> BoxFuture<jsonrpc_core::Result<Vec<Group>>>;

    #[rpc(meta, name = "all_groups_get", params = "named")]
    fn all_groups_get(&self, meta: Self::Metadata) -> BoxFuture<jsonrpc_core::Result<Vec<Group>>>;

    #[rpc(meta, name = "groups_join", params = "named")]
    fn groups_join(
        &self,
        meta: Self::Metadata,
        groups: Vec<i32>,
    ) -> BoxFuture<jsonrpc_core::Result<String>>;

    #[rpc(meta, name = "schedule_get", params = "named")]
    fn schedule_get(
        &self,
        meta: Self::Metadata,
        start: NaiveDateTime,
        end: NaiveDateTime,
        group: i32,
    ) -> BoxFuture<jsonrpc_core::Result<Vec<Course>>>;

    #[rpc(meta, name = "client_configs_get", params = "named")]
    fn client_configs_get(
        &self,
        meta: Self::Metadata,
        client_id: i32,
    ) -> BoxFuture<jsonrpc_core::Result<Option<String>>>;

    #[rpc(meta, name = "client_configs_set", params = "named")]
    fn client_configs_set(
        &self,
        meta: Self::Metadata,
        client_id: i32,
        config: String,
    ) -> BoxFuture<jsonrpc_core::Result<String>>;

    #[rpc(name = "send_password_reset_code", params = "named")]
    fn send_password_reset_code(
        &self,
        ldap: i64,
        email: String,
    ) -> BoxFuture<jsonrpc_core::Result<String>>;

    #[rpc(name = "reset_password", params = "named")]
    fn reset_password(
        &self,
        code: String,
        password: String,
    ) -> BoxFuture<jsonrpc_core::Result<String>>;
}

pub struct RpcImpl(Arc<RpcState>);

struct RpcState {
    db: PgPool,
    // FIXME: put those in the DB
    new_users_tokens: Mutex<HashMap<String, User>>,
    reset_password_tokens: Mutex<HashMap<String, User>>,
    mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl RpcImpl {
    pub fn new(db: PgPool) -> Result<RpcImpl, lettre::transport::smtp::Error> {
        Ok(RpcImpl(Arc::new(RpcState {
            db,
            new_users_tokens: Mutex::new(HashMap::new()),
            reset_password_tokens: Mutex::new(HashMap::new()),
            mailer: AsyncSmtpTransport::<Tokio1Executor>::relay(&SETTINGS.smtp.server)?
                .credentials(Credentials::new(
                    SETTINGS.smtp.username.clone(),
                    SETTINGS.smtp.password.clone(),
                ))
                .build(),
        })))
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

    fn time(&self) -> jsonrpc_core::Result<NaiveDateTime> {
        info!("time");
        Ok(Utc::now().naive_utc())
    }

    fn login(&self, email: String, password: String) -> BoxFuture<jsonrpc_core::Result<String>> {
        let state = Arc::clone(&self.0);
        Box::pin(async move {
            let user: User = match server_error! {
                sqlx::query_as!(User, "select * from users where email = $1", email)
                    .fetch_optional(&state.db)
                    .await
            } {
                Some(user) => user,
                None => {
                    warn!("{} isn't a know email", email);
                    return Err(RpcError::IncorrectLoginInfo.into());
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
        let state = Arc::clone(&self.0);
        Box::pin(async move {
            let dpmt: Department = match server_error! {
                sqlx::query_as!(Department, "select * from departments where id = $1", department)
                    .fetch_optional(&state.db)
                    .await
            } {
                Some(dpmt) => dpmt,
                None => {
                    warn!("department {} is unknown", department);
                    return Err(RpcError::UnknownDepartment.into());
                }
            };

            let mut email = email;
            email.push_str("@");
            email.push_str(&*dpmt.domain);

            if server_error!(
                sqlx::query!("select id from users where id = $1", ldap)
                    .fetch_optional(&state.db)
                    .await
            )
            .is_some()
            {
                warn!("user {} is already registered", ldap);
                return Err(RpcError::AlreadyRegistered.into());
            }

            match server_error! {
                sqlx::query!("select id from users where email = $1", email)
                    .fetch_optional(&state.db)
                    .await
            } {
                Some(x) => {
                    warn!("email {} is already used for user {}", email, x.id);
                    return Err(RpcError::AlreadyRegistered.into());
                }
                None => {}
            }

            let (firstname, lastname) = match server_error! {
                sqlx::query!(
                    "select firstname, lastname from celcat_students where id = $1 and department = $2",
                    ldap, department
                ).fetch_optional(&state.db).await
            } {
                Some(x) => (x.firstname, x.lastname),
                None => {
                    warn!("User {} in department: {} is unknown", ldap, department);
                    return Err(RpcError::IncorrectLoginInfo.into());
                }
            };

            let hash = uuid::Uuid::new_v4().to_string();
            debug!("hash: {}", hash);

            let user = User {
                id: ldap,
                firstname,
                lastname,
                email: email.clone(),
                password: "".to_string(),
            };
            state
                .new_users_tokens
                .lock()
                .unwrap()
                .insert(hash.clone(), user);

            let message = match email::gen_inscription(&email, &hash) {
                Ok(msg) => msg,
                Err(err) => {
                    warn!("{}", err);
                    return Err(RpcError::UnknownError.into());
                }
            };
            match state.mailer.send(message).await {
                Ok(_) => Ok("Code sent".to_string()),
                Err(err) => {
                    warn!("{}", err);
                    return Err(RpcError::UnknownError.into());
                }
            }
        })
    }

    fn register_2(&self, hash: String) -> BoxFuture<jsonrpc_core::Result<Identity>> {
        let state = Arc::clone(&self.0);
        Box::pin(async move {
            match state.new_users_tokens.lock().unwrap().get(&hash) {
                Some(user) => Ok(Identity {
                    firstname: user.firstname.clone(),
                    lastname: user.lastname.clone(),
                }),
                None => {
                    warn!(
                        "Someone tried to use an used or inexistant token: {}",
                        hash.to_owned()
                    );
                    Err(RpcError::RegistrationTokenUsed.into())
                }
            }
        })
    }

    fn register_3(
        &self,
        hash: String,
        firstname: String,
        lastname: String,
        password: String,
    ) -> BoxFuture<jsonrpc_core::Result<String>> {
        let state = Arc::clone(&self.0);
        Box::pin(async move {
            let user = match state.new_users_tokens.lock().unwrap().remove(&hash) {
                Some(u) => User {
                    firstname,
                    lastname,
                    password: authentication::hash_password(password, u.id.to_string()),
                    ..u
                },
                None => {
                    warn!(
                        "Someone tried to use an used or inexistant token: {}",
                        hash.to_owned()
                    );
                    return Err(RpcError::RegistrationTokenUsed.into());
                }
            };

            server_error! {
                sqlx::query!(
                    "insert into users (id, firstname, lastname, email, password)
                     values ($1, $2, $3, $4, $5)",
                    user.id, user.firstname, user.lastname, user.email, user.password,
                ).execute(&state.db).await
            };

            Ok("Account created!".to_string())
        })
    }

    fn is_logged(&self, meta: Self::Metadata) -> BoxFuture<jsonrpc_core::Result<bool>> {
        let state = Arc::clone(&self.0);
        Box::pin(async move {
            let user = authentication::logged_user_get(&state.db, meta).await;
            return match user {
                Some(_) => Ok(true),
                None => Ok(false),
            };
        })
    }

    fn my_groups_get(&self, meta: Self::Metadata) -> BoxFuture<jsonrpc_core::Result<Vec<Group>>> {
        let state = Arc::clone(&self.0);
        Box::pin(async move {
            match authentication::logged_user_get(&state.db, meta).await {
                Some(user) => Ok(server_error! {
                    sqlx::query_as!(
                        Group,
                        "select g.* from groups as g
                         join users_groups as ug on ug.group_id = g.id
                         where ug.user_id = $1",
                        user.id,
                    ).fetch_all(&state.db).await
                }),
                None => Err(RpcError::IncorrectLoginInfo.into()),
            }
        })
    }

    fn all_groups_get(&self, meta: Self::Metadata) -> BoxFuture<jsonrpc_core::Result<Vec<Group>>> {
        let state = Arc::clone(&self.0);
        Box::pin(async move {
            if authentication::logged_user_get(&state.db, meta)
                .await
                .is_none()
            {
                return Err(RpcError::IncorrectLoginInfo.into());
            }
            Ok(server_error! {
                sqlx::query_as!(Group, "select * from groups where private = false")
                    .fetch_all(&state.db).await
            })
        })
    }

    fn groups_join(
        &self,
        meta: Self::Metadata,
        groups: Vec<i32>,
    ) -> BoxFuture<jsonrpc_core::Result<String>> {
        let state = Arc::clone(&self.0);
        Box::pin(async move {
            match authentication::logged_user_get(&state.db, meta).await {
                Some(user) => {
                    for group in groups {
                        server_error! {
                            sqlx::query!(
                                "insert into users_groups (user_id, group_id)
                                 select $1, $2
                                 from groups where id = $2 and private = false
                                 on conflict (user_id, group_id) do nothing",
                                user.id, group,
                            ).execute(&state.db).await
                        };
                    }
                    Ok("Success!".to_string())
                }
                None => Err(RpcError::IncorrectLoginInfo.into()),
            }
        })
    }

    fn schedule_get(
        &self,
        meta: Self::Metadata,
        start: NaiveDateTime,
        end: NaiveDateTime,
        group: i32,
    ) -> BoxFuture<jsonrpc_core::Result<Vec<Course>>> {
        let state = Arc::clone(&self.0);
        Box::pin(async move {
            match authentication::logged_user_get(&state.db, meta).await {
                Some(user) => match server_error! {
                    sqlx::query!(
                        "select from users_groups where user_id = $1 and group_id = $2",
                        user.id, group,
                    ).fetch_optional(&state.db).await
                } {
                    Some(_) => Ok(server_error! {
                        sqlx::query_as!(
                            Course,
                            "select c.* from courses as c
                             join groups_courses as gc on c.id = gc.course_id
                             where gc.group_id = $1 and c.start_time >= $2 and c.end_time <= $3",
                            group, start, end,
                        ).fetch_all(&state.db).await
                    }),
                    None => Err(RpcError::Unimplemented.into()),
                },
                None => Err(RpcError::IncorrectLoginInfo.into()),
            }
        })
    }

    fn client_configs_get(
        &self,
        meta: Self::Metadata,
        client_id: i32,
    ) -> BoxFuture<jsonrpc_core::Result<Option<String>>> {
        let state = Arc::clone(&self.0);
        Box::pin(async move {
            let user = match authentication::logged_user_get(&state.db, meta).await {
                Some(user) => user,
                None => {
                    return Err(RpcError::IncorrectLoginInfo.into());
                }
            };

            if server_error!(
                sqlx::query!("select from clients where id = $1", client_id)
                    .fetch_optional(&state.db)
                    .await
            )
            .is_none()
            {
                return Err(RpcError::UnknownClient.into());
            }

            Ok(server_error!(
                sqlx::query!(
                    "select config from clients_users_config
                     where client_id = $1 and user_id = $2",
                    client_id,
                    user.id,
                )
                .fetch_optional(&state.db)
                .await
            )
            .and_then(|x| x.config))
        })
    }

    fn client_configs_set(
        &self,
        meta: Self::Metadata,
        client_id: i32,
        config: String,
    ) -> BoxFuture<jsonrpc_core::Result<String>> {
        let state = Arc::clone(&self.0);
        Box::pin(async move {
            let user = match authentication::logged_user_get(&state.db, meta).await {
                Some(user) => user,
                None => {
                    return Err(RpcError::IncorrectLoginInfo.into());
                }
            };

            if server_error!(
                sqlx::query!("select from clients where id = $1", client_id)
                    .fetch_optional(&state.db)
                    .await
            )
            .is_none()
            {
                return Err(RpcError::UnknownClient.into());
            }

            server_error! {
                sqlx::query!(
                    "insert into clients_users_config (client_id, user_id, config)
                     values ($1, $2, $3)
                     on conflict (client_id, user_id) do update set config = excluded.config",
                    client_id,
                    user.id,
                    config,
                )
                .execute(&state.db)
                .await
            };

            Ok("Success!".to_string())
        })
    }

    fn send_password_reset_code(
        &self,
        ldap: i64,
        email: String,
    ) -> BoxFuture<jsonrpc_core::Result<String>> {
        let state = Arc::clone(&self.0);
        Box::pin(async move {
            let user = match server_error! {
                sqlx::query_as!(User, "select * from users where id = $1", ldap)
                    .fetch_optional(&state.db)
                    .await
            } {
                Some(user) => user,
                None => {
                    return Err(RpcError::IncorrectLoginInfo.into());
                }
            };

            if user.email != email {
                return Err(RpcError::IncorrectLoginInfo.into());
            }

            let hash = uuid::Uuid::new_v4().to_string();
            debug!("hash: {}", hash);

            let firstname = user.firstname.clone();
            let lastname = user.lastname.clone();

            state
                .reset_password_tokens
                .lock()
                .unwrap()
                .insert(hash.clone(), user);

            let message = match email::gen_reset(&email, &firstname, &lastname, &hash) {
                Ok(msg) => msg,
                Err(err) => {
                    warn!("{}", err);
                    return Err(RpcError::UnknownError.into());
                }
            };
            match state.mailer.send(message).await {
                Ok(_) => Ok("Code sent".to_string()),
                Err(err) => {
                    warn!("{}", err);
                    return Err(RpcError::UnknownError.into());
                }
            }
        })
    }

    fn reset_password(
        &self,
        code: String,
        password: String,
    ) -> BoxFuture<jsonrpc_core::Result<String>> {
        let state = Arc::clone(&self.0);
        Box::pin(async move {
            let user = match state.reset_password_tokens.lock().unwrap().remove(&code) {
                Some(u) => User {
                    password: authentication::hash_password(password.clone(), u.id.to_string()),
                    ..u
                },
                None => {
                    warn!(
                        "Someone tried to use a used or inexistant token: {}",
                        code.to_owned()
                    );
                    return Err(RpcError::Unimplemented.into());
                }
            };

            server_error! {
                sqlx::query!(
                    "update users set password = $1 where id = $2",
                    password, user.id,
                ).execute(&state.db).await
            };

            Ok("Password changed!".to_string())
        })
    }
}
