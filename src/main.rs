use futures::future;
use jsonrpc_core::*;
use jsonrpc_http_server::*;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone)]
struct Auth {
    jwt: Option<String>,
}

impl Metadata for Auth {}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

fn main() {
    let mut io = MetaIoHandler::default();

    io.add_method("ping", |_params: Params| {
        future::ok(Value::String("pong".to_owned()))
    });

    io.add_method_with_meta("test", |_params: Params, auth: Auth| {
        let jwt = match auth.jwt {
            Some(s) => s,
            None => {
                return future::err(Error::invalid_request());
            }
        };
        let token_data = match decode::<Claims>(
            &jwt,
            &DecodingKey::from_secret(b"secret"),
            &Validation::new(Algorithm::HS256),
        ) {
            Ok(c) => c,
            Err(e) => {
                return future::err(Error {
                    code: ErrorCode::ServerError(3),
                    message: "invalid jwt".to_owned(),
                    data: Some(Value::String(e.to_string())),
                });
            }
        };
        future::ok(to_value(token_data.claims).unwrap())
    });

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
            Auth { jwt }
        })
        .start_http(&"127.0.0.1:3030".parse().unwrap())
        .unwrap();

    server.wait();
}
