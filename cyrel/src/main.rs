mod authentication;
mod email;
mod groups;
mod models;
mod rpc;
mod schedule;
mod settings;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use anyhow::anyhow;
use clap::{clap_app, crate_authors, crate_description, crate_name, crate_version, ArgMatches};
use dotenv::dotenv;
use jsonrpc_core::*;
use jsonrpc_http_server::*;
use lazy_static::lazy_static;
use sqlx::postgres::PgPoolOptions;
use tracing::{debug, info, trace};
use tracing_subscriber::EnvFilter;

use crate::authentication::Meta;
use crate::rpc::{gen_server::Rpc, RpcImpl};
use crate::settings::Settings;

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init()
        .map_err(|e| anyhow!(e))?;

    lazy_static::initialize(&CLI);
    lazy_static::initialize(&SETTINGS);

    debug!("{:#?}", *SETTINGS);

    let mut io = MetaIoHandler::default();

    let db = PgPoolOptions::new().connect(&SETTINGS.database.url).await?;
    let rpc = RpcImpl::new(db).unwrap();

    io.extend_with(rpc.to_delegate());

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), SETTINGS.port);

    let server = ServerBuilder::new(io)
        .cors_allow_headers(cors::AccessControlAllowHeaders::Any)
        .meta_extractor(|req: &hyper::Request<hyper::Body>| {
            let jwt = req
                .headers()
                .get(hyper::header::AUTHORIZATION)
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.strip_prefix("Bearer ")) // FIXME: reliable?
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

    Ok(())
}
