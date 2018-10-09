#![feature(plugin)]
#![plugin(tarpc_plugins)]
#![allow(proc_macro_derive_resolution_fallback)]
#![feature(try_from)]
#![feature(crate_in_paths)]
#![feature(extern_prelude)]

pub mod db;
pub mod error;
pub mod logging;
pub mod migration;
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
extern crate futures;
extern crate futures_cpupool;
extern crate pbkdf2;
extern crate rand;

use dotenv::dotenv;
use error::{Error as IntError, ErrorKind as IntErrorKind};
use failure::Error;
use service::FutureServiceExt;
use std::net::{SocketAddr, ToSocketAddrs};
use tarpc::future::server::Options;
use tokio_core::reactor;

type IntResult<T> = Result<T, IntError>;

pub fn run() -> Result<(), Error> {
    // Get command line arguments
    let cmd_arguments = clap::App::new("auth-service")
        .arg(
            clap::Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Increases logging verbosity each use for up to 3 times"),
        ).arg(
            clap::Arg::with_name("migrate")
                .short("m")
                .long("migrate")
                .multiple(true)
                .help("Runs db migration"),
        ).get_matches();

    // Setup logging
    let verbosity: u64 = cmd_arguments.occurrences_of("verbose");

    logging::setup_logging(verbosity)
        .map_err(|e| format_err!("failed to initialize logging: {:?}", e))?;

    // Create an "eventloop"
    let mut reactor = reactor::Core::new()
        .map_err(|e| format_err!("unable to create a tokio runtime: {:?}", e))?;

    // Get db url
    dotenv().ok();
    let database_url = std::env::var("AUTH_DATABASE_URL")
        .expect("AUTH_DATABASE_URL must be set as an environment variable or in a '.env' file");

    // Get the server address
    let address = match std::env::var("AUTH_ADDRESS") {
        Ok(value) => value
            .to_socket_addrs()
            .expect("Unable to perorm AUTH_ADDRESS resolving")
            .next()
            .expect(&format!("Unable to resolve '{}'", value)),
        Err(_) => {
            warn!("AUTH_ADDRESS is not set, using '127.0.0.1:10001'");
            SocketAddr::from(([127, 0, 0, 1], 10001))
        }
    };

    // Setup server
    info!("Setting up server");
    let auth_server = service::AuthServer::try_new(&database_url)?;

    //Migrate
    let migrate: u64 = cmd_arguments.occurrences_of("migrate");
    if migrate > 0 {
        info!("Running db migration");
        migration::run(&database_url)?;
    }

    // Start
    let opts = Options::default();
    let (_handle, server) = auth_server
        .listen(address, &reactor.handle(), opts)
        .map_err(|e| format_err!("Unable to startup server: {:?}", e))?;

    info!("Starting server on {}", address);
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
