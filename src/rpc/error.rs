use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum RpcError {
    #[error("incorrect login information")]
    IncorrectLoginInfo = 1,
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
