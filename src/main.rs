#![feature(plugin)]
#![plugin(tarpc_plugins)]

mod pass_funcs;
mod logging;

#[macro_use]
extern crate tarpc;
extern crate serde;
extern crate serde_json;
extern crate regex;
extern crate chrono;

#[macro_use]
extern crate serde_derive;

use regex::Regex;
use serde_json::{Value,Error};

#[derive(Serialize, Deserialize)]
struct JsonData {
    data_type: String,
    payload: Payload,
}

#[derive(Serialize, Deserialize)]
struct Payload {
    username: String,
    // password: String
}

fn parse_json_data(s: &str) -> Result<Value, Error> {
    // Parse the string of data into serde_json::Value.
    let v: Value = serde_json::from_str(&s)?;
    Ok(v)
}

fn strip_characters(original : String, to_strip : &str) -> String {
    let mut result = String::new();
    for c in original.chars() {
        if !to_strip.contains(c) {
           result.push(c);
       }
    }
    result
}

// I URAN, AM NOT PROUD OF THIS FUNCTION !
// password - letters (lowercase AND uppercase) AND numbers, 8-50 characters
fn check_pass_requirement(input: &str) -> bool {
    
    // check password length
    // check if password has numbers, then check for lowercase letters, then check for uppercase letters
    // finally, check there are no special characters?
    if input.chars().count() >= 8 && input.chars().count() <=50 {
        let re = Regex::new(r"[0-9]+").unwrap();
        if re.is_match(&input) {
            let re = Regex::new(r"[a-z]+").unwrap();
            if re.is_match(&input) {
                let re = Regex::new(r"[A-Z]+").unwrap();
                if re.is_match(&input) {
                    let re = Regex::new(r"^[a-zA-Z0-9]+$").unwrap();
                    if re.is_match(&input) {
                        return true;
                    }
                }
            }
        }
    }
    return false;
}

use std::sync::mpsc;
use std::thread;
use tarpc::sync::{client, server};
use tarpc::sync::client::ClientExt;
use tarpc::util::{FirstSocketAddr, Never};

service! {
    rpc hello(name: String) -> String;
}

#[derive(Clone)]
struct HelloServer;

impl SyncService for HelloServer {
    fn hello(&self, name: String) -> Result<String, Never> {
        Ok(format!("Hello, {}!", name))
    }
}

// Authentication part - TO DO
fn authenticate() {

}

fn main() {

    // Some variables
    let request : String;
    let username : String;
    let password : String;
    let hashed_password : String;

    // This is easier to work with
    let json_data = r#"{
        "type": "Authenticate",
        "username": "james",
        "password": "TestPass321"
    }"#;

    // serde_json::from_str seems to return a Result type
    // This is the only way I could get Result<T, E>
    // to work while returning the parsed JSON data to main()
    match parse_json_data(json_data) {
        Ok(_v) => {
            request = _v["type"].to_string();
            username = _v["username"].to_string();
            password = _v["password"].to_string();
        },
        Err(_v) => {
            request = "".to_string();
            username = "".to_string();
            password = "".to_string();
        }
    }

    // Remove the parentheses from JSON data received
    let request = strip_characters(request, r#"""#);
    let username = strip_characters(username, r#"""#);
    let password = strip_characters(password, r#"""#);

    // Add log info to the log file
    logging::create_log(&request, &username);

    // password - letters (lowercase AND uppercase) AND numbers, 8-50 characters
    // RegEx in Rust doesn't support look ahead assertion, so the RegEx would be very complicated
    if check_pass_requirement(&password) {
        println!("Password matches requirements");
    } else {
        println!("Password doesn't match requirements");
        return;
    }

    // Hash the password
    hashed_password = pass_funcs::hash_password(&password);

    // Check if hashed password matches entered password
    if pass_funcs::check_password(&password, &hashed_password) {
        println!("Password matches");
    } else {
        println!("Password doesn't match");
    }

    println!("{} {} {} \n{}",request, username, password, hashed_password);

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let handle = HelloServer.listen("localhost:0", server::Options::default())
            .unwrap();
        tx.send(handle.addr()).unwrap();
        handle.run();
    });
    let client = SyncClient::connect(rx.recv().unwrap(), client::Options::default()).unwrap();
    println!("{}", client.hello("Mom".to_string()).unwrap());
}