#[macro_use]
extern crate diesel;

mod authentication;
mod celcat;
mod groups;
mod models;
mod rpc;
mod schedule;
mod schema;
mod settings;
mod users;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};

use clap::{clap_app, crate_authors, crate_description, crate_name, crate_version, ArgMatches};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use jsonrpc_core::*;
use jsonrpc_http_server::*;
use lazy_static::lazy_static;
use log::{debug, info, trace};
use rand::prelude::*;

use crate::authentication::{Meta, Register};
use crate::models::User;
use crate::rpc::{gen_server::Rpc, RpcImpl};
use crate::settings::Settings;
use std::collections::HashMap;

lazy_static! {
    static ref CLI: ArgMatches<'static> = clap_app!(
        cyrel =>
            (name: crate_name!())
            (version: crate_version!())
            (author: crate_authors!())
            (about: crate_description!())
            (@arg CONFIG: -c --config +takes_value "config file to read")
            (@arg PORT: -p --port +takes_value "port to use")
    )
    .get_matches();
    static ref SETTINGS: Settings = Settings::new(&CLI).expect("failed to read settings");
}

fn main() {
    dotenv().ok();
    env_logger::init();

    lazy_static::initialize(&CLI);
    lazy_static::initialize(&SETTINGS);

    debug!("{:#?}", *SETTINGS);

    let db_conn = SqliteConnection::establish(&SETTINGS.database.url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", SETTINGS.database.url));
    info!("connected to the database");

    let mut io = MetaIoHandler::default();
    let rpc = RpcImpl {
        db: Arc::new(Mutex::new(db_conn)),
        rng: StdRng::from_entropy(),
        register: Arc::new(Mutex::new(Register {
            tokens: HashMap::<String, User>::new(),
        })),
    };

    io.extend_with(rpc.to_delegate());

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), SETTINGS.port);

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
        .start_http(&addr)
        .unwrap();

    info!("rpc started at {}", addr);

    server.wait();
}
