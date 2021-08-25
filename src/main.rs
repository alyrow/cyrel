#[macro_use]
extern crate diesel;

mod authentication;
mod models;
mod rpc;
mod schema;

use std::env;
use std::sync::{Arc, Mutex};

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;

use jsonrpc_core::*;
use jsonrpc_http_server::*;
use log::{debug, info, trace};
use rand::prelude::*;

use crate::authentication::Meta;
use crate::rpc::{gen_server::Rpc, RpcImpl};

fn main() {
    dotenv().ok();
    env_logger::init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    debug!("DATABASE_URL={}", db_url);
    let db_conn =
        SqliteConnection::establish(&db_url).unwrap_or_else(|_| panic!("Error connecting to {}", db_url));
    info!("connected to the database");

    // FIXME: make it random (more difficult for debugging)
    let jwt_secret = Box::leak(
        env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set")
            .into_boxed_str(),
    );
    debug!("JWT_SECRET={}", jwt_secret);

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
            trace!("got JWT: {:?}", jwt);
            Meta { jwt }
        })
        .request_middleware(|req: hyper::Request<hyper::Body>| {
            trace!("{:?}", req);
            req.into()
        })
        .start_http(&"127.0.0.1:3030".parse().unwrap())
        .unwrap();

    info!("rpc started");

    server.wait();
}
