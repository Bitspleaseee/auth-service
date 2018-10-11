#![feature(plugin)]
#![feature(try_from)]
#![feature(try_trait)]
#![plugin(tarpc_plugins)]

#[macro_use]
extern crate tarpc;
extern crate rustyline;
extern crate serde_derive;

extern crate datatypes;
#[macro_use]
extern crate failure;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::convert::{TryFrom, TryInto};
use std::default::Default;
use std::fmt::Debug;
use std::net::{SocketAddr, ToSocketAddrs};

use tarpc::sync::client;
use tarpc::sync::client::ClientExt;
use tarpc::util::FirstSocketAddr;

use datatypes::auth::requests::*;
use datatypes::auth::responses::*;
use datatypes::content::requests::AddUserPayload;
use datatypes::payloads::*;
use datatypes::valid::fields::*;
use datatypes::valid::ids::*;
use datatypes::valid::token::Token;

use failure::Error;
use failure::Fallible;

service! {
    rpc authenticate(payload: AuthPayload) -> Token | AuthError;
    rpc deauthenticate(payload: Token) -> () | AuthError;
    rpc register(payload: RegisterUserPayload) -> AddUserPayload | AuthError;
    rpc get_user_role(payload: Token) -> Role | AuthError;
    rpc set_user_role(payload: SetUserRolePayload) -> () | AuthError;
}

#[derive(Copy, Clone)]
pub enum Cmd {
    Auth,
    Deauth,
    Register,
    SetRole,
}

impl<'a> TryFrom<&'a str> for Cmd {
    type Error = Error;
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        use self::Cmd::*;
        match s {
            "auth" => Ok(Auth),
            "deauth" => Ok(Deauth),
            "register" => Ok(Register),
            "set-role" => Ok(SetRole),
            e => Err(format_err!("Invalid command: {}", e)),
        }
    }
}

#[derive(Copy, Clone)]
pub enum Mode {
    Main,
}

impl<'a> TryFrom<&'a str> for Mode {
    type Error = Error;
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        use self::Mode::*;
        match s {
            "main" => Ok(Main),
            e => Err(format_err!("Invalid mode: {}", e)),
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

fn cmd_handler<'a>(state: &State, s: &'a str) -> Fallible<()> {
    let mut args = s.split(char::is_whitespace);
    let cmd = args
        .next()
        .ok_or(format_err!("Missing argument <cmd>"))
        .and_then(|w| w.try_into())?;
    match (state.mode, cmd) {
        (Mode::Main, Cmd::Auth) => run_auth(args),
        (Mode::Main, Cmd::Deauth) => run_deauth(args),
        (Mode::Main, Cmd::Register) => run_register(args),
        (Mode::Main, Cmd::SetRole) => run_set_user_role(args),
    }
}

macro_rules! get_next_arg {
    ($args:ident, $name:ident) => {
        $args
            .next()
            .ok_or(format_err!("Missing argument <{}>", stringify!($name)))
            .and_then(|s| {
                s.to_owned()
                    .try_into()
                    .map_err(|_| format_err!("Invalid <{}>", stringify!($name)))
            })
    };
}
macro_rules! get_next_id {
    ($args:ident, $inner:ty => $name:ident) => {
        $args
            .next()
            .ok_or(format_err!("Missing argument <{}>", stringify!($name)))
            .and_then(|s| {
                s.parse::<$inner>()
                    .map(|n| n.into())
                    .map_err(|_| format_err!("Invalid <{}>", stringify!($name)))
            })
    };
}

// Auth
fn run_auth<'a>(mut args: impl Iterator<Item = &'a str>) -> Fallible<()> {
    let username = get_next_arg!(args, username)?;
    let password = get_next_arg!(args, password)?;
    let payload = AuthPayload { username, password };

    run_client_action(|client| client.authenticate(payload));

    Ok(())
}

fn run_deauth<'a>(mut args: impl Iterator<Item = &'a str>) -> Fallible<()> {
    unimplemented!()
}

fn run_register<'a>(mut args: impl Iterator<Item = &'a str>) -> Fallible<()> {
    let username = get_next_arg!(args, username)?;
    let password = get_next_arg!(args, password)?;
    let email = get_next_arg!(args, email)?;
    let payload = RegisterUserPayload {
        username,
        password,
        email,
    };
    run_client_action(|client| client.register(payload));
    Ok(())
}

fn run_set_user_role<'a>(mut args: impl Iterator<Item = &'a str>) -> Fallible<()> {
    let id = get_next_id!(args, u32 => user_id)?;

    let role: String = get_next_arg!(args, role)?;
    let role = (&*role).into();

    let payload = SetUserRolePayload { id, role };
    run_client_action(|client| client.set_user_role(payload));
    Ok(())
}

// Connect to server
fn connect() -> Option<SyncClient> {
    let address = match std::env::var("AUTH_ADDRESS") {
        Ok(value) => value
            .to_socket_addrs()
            .expect("Unable to perorm AUTH_ADDRESS resolving")
            .next()
            .expect(&format!("Unable to resolve '{}'", value)),
        Err(_) => {
            println!("AUTH_ADDRESS is not set, using '127.0.0.1:10001'");
            SocketAddr::from(([127, 0, 0, 1], 10001))
        }
    };

    let options = client::Options::default();
    SyncClient::connect(address, options).ok()
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
    } else {
        println!("Unable to connect");
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
                    cmd_handler(&state, &line).unwrap_or_else(|err| println!("Error: {}", err));
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt").unwrap();
}
