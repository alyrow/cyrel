use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum RpcError {
    #[error("unknown error")]
    UnknownError = 0,

    #[error("incorrect login information")]
    IncorrectLoginInfo = 1,

    #[error("already registered")]
    AlreadyRegistered = 3,

    #[error("the token was already used")]
    RegistrationTokenUsed = 4,

    #[error("unimplemented")]
    Unimplemented = 2,
}

impl From<RpcError> for jsonrpc_core::Error {
    fn from(r: RpcError) -> Self {
        Self {
            code: jsonrpc_core::ErrorCode::ServerError(r.clone() as i64),
            message: r.to_string(),
            data: None, // TODO: enum data
        }
    }
}
