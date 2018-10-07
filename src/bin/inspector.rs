#![feature(plugin)]
#![feature(try_from)]
#![feature(try_trait)]
#![plugin(tarpc_plugins)]

#[macro_use]
extern crate tarpc;
extern crate rustyline;
extern crate serde_derive;

extern crate datatypes;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::convert::{TryFrom, TryInto};
use std::default::Default;
use std::fmt::Debug;

use tarpc::sync::client;
use tarpc::sync::client::ClientExt;
use tarpc::util::FirstSocketAddr;

use datatypes::auth::requests::*;
use datatypes::auth::responses::*;
use datatypes::valid::fields::*;
use datatypes::valid::token::Token;
use datatypes::payloads::*;

service! {
    rpc authenticate(payload: AuthPayload) -> Token | AuthError;
    rpc deauthenticate(payload: TokenPayload<EmptyPayload>) -> () | AuthError;
    /* more services should be defined (deauthenticate etc) */
}

#[derive(Copy, Clone)]
pub enum Cmd {
    Auth,
    Deauth,
}

impl TryFrom<&str> for Cmd {
    type Error = ();
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        use self::Cmd::*;
        match s {
            "auth" => Ok(Auth),
            "deauth" => Ok(Deauth),
            _ => Err(()),
        }
    }
}

#[derive(Copy, Clone)]
pub enum Mode {
    Main,
}

impl TryFrom<&str> for Mode {
    type Error = ();
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        use self::Mode::*;
        match s {
            "main" => Ok(Main),
            _ => Err(()),
        }
    }
}

impl Mode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Mode::Main => "main",
        }
    }
}

fn cmd_handler<'a>(state: &State, s: &'a str) -> Result<(), std::option::NoneError> {
    let mut words = s.split(char::is_whitespace);
    let cmd = words.next().and_then(|w| w.try_into().ok())?;
    match (state.mode, cmd) {
        (Mode::Main, Cmd::Auth) => run_auth(words),
        (Mode::Main, Cmd::Deauth) => run_deauth(words),
    }
}

// Auth
fn run_auth<'a>(mut args: impl Iterator<Item = &'a str>) -> Result<(), std::option::NoneError> {
    let username = args.next().and_then(|s| {
        s.to_owned()
            .try_into()
            .map_err(|e| {
                eprintln!("{:?}", e);
                e
            }).ok()
    })?;
    let password = args.next().and_then(|s| {
        s.to_owned()
            .try_into()
            .map_err(|e| {
                eprintln!("{:?}", e);
                e
            }).ok()
    })?;
    let payload = AuthPayload { username, password };

    run_client_action(|client| client.authenticate(payload));
    Ok(())
}

fn run_deauth<'a>(mut args: impl Iterator<Item = &'a str>) -> Result<(), std::option::NoneError> {
    unimplemented!()
}

// Connect to server
fn connect() -> Option<SyncClient> {
    let options = client::Options::default();
    let addr = "localhost:10001".first_socket_addr();

    SyncClient::connect(addr, options)
        .map_err(|e| println!("Error connecting: {:#?}", e))
        .ok()
}

// Run a action on the server and print the result
fn run_client_action<T, E, F>(f: F)
where
    T: Debug,
    E: Debug,
    F: FnOnce(SyncClient) -> Result<T, E>,
{
    if let Some(client) = connect() {
        match f(client) {
            Ok(value) => println!("The server responded with: {:#?}", value),
            Err(error) => println!("The server responded with error: {:#?}", error),
        }
    }
}

pub struct State {
    mode: Mode,
}

impl State {
    pub fn try_set_mode(&mut self, maybe_mode: impl TryInto<Mode>) -> Option<Mode> {
        maybe_mode
            .try_into()
            .ok()
            .map(|new_mode| std::mem::replace(&mut self.mode, new_mode))
    }
}

impl Default for State {
    fn default() -> State {
        State { mode: Mode::Main }
    }
}

fn main() {
    let mut state = State::default();

    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline(&format!("{} >> ", state.mode.as_str()));
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_ref());
                // Change between databases ol.
                if line.starts_with("mode") {
                    line.split(char::is_whitespace)
                        .nth(1)
                        .map(|mode_str| state.try_set_mode(mode_str));
                } else {
                    cmd_handler(&state, &line).unwrap_or_else(|err| println!("Error: {:?}", err));
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt").unwrap();
}
