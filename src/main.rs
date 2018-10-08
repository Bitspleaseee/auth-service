#![feature(plugin)]
#![plugin(tarpc_plugins)]
#![allow(proc_macro_derive_resolution_fallback)]
#![feature(try_from)]
#![feature(crate_in_paths)]
#![feature(extern_prelude)]

pub mod db;
pub mod error;
pub mod logging;
pub mod schema;
pub mod service;
#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate tarpc;
extern crate chrono;
extern crate clap;
extern crate regex;
extern crate serde;
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate fern;
extern crate tokio_core;
#[macro_use]
extern crate failure;
extern crate base64;
extern crate datatypes;
extern crate pbkdf2;
extern crate rand;
use error::{Error as IntError, ErrorKind as IntErrorKind};
use failure::Error;
use service::FutureServiceExt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tarpc::future::server::Options;
use tokio_core::reactor;

type IntResult<T> = Result<T, IntError>;
pub fn run() -> Result<(), Error> {
    // Get verbosity of program from the commandline
    let cmd_arguments = clap::App::new("auth-service")
        .arg(
            clap::Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Increases logging verbosity each use for up to 3 times"),
        ).get_matches();

    let verbosity: u64 = cmd_arguments.occurrences_of("verbose");

    logging::setup_logging(verbosity)
        .map_err(|e| format_err!("failed to initialize logging: {:?}", e))?;

    // Create an "eventloop"
    let mut reactor = reactor::Core::new()
        .map_err(|e| format_err!("unable to create a tokio runtime: {:?}", e))?;

    // Create a server with a default state (e.g empty HashMap)
    let auth_server = service::AuthServer::default();

    // TODO set addr from environmen variables
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 10001);
    let opts = Options::default();

    let (_handle, server) = auth_server
        .listen(addr, &reactor.handle(), opts)
        .map_err(|e| format_err!("Unable to startup server: {:?}", e))?;

    info!("starting up server on {}", addr);
    reactor
        .run(server)
        .map_err(|_| format_err!("quit from eventloop with error"))
}

pub fn main() {
    if let Err(e) = run() {
        error!("Exit with error: {:?}", e);
        std::process::exit(1)
    } else {
        std::process::exit(0)
    }
}
