extern crate serde;
extern crate serde_json;
extern crate pbkdf2;
extern crate regex;
extern crate chrono;

#[macro_use]
extern crate serde_derive;

use regex::Regex;
use serde_json::{Value,Error};
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;
use std::fs::File;

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


// Authentication part - TO DO
fn authenticate() {

}

// Create log for login/register attempt
fn create_log(request : &str, username: &str) {

    let file_name = "logFile";

    // Check if file exists
    // If it doesn't, create it
    let file_exists : bool = Path::new(file_name).exists();
    if !file_exists {
        if let Err(e) = File::create(file_name) {
            eprintln!("Couldn't create file: {}", e);
            return;
        }
    }

    // Document the time
    let now = chrono::Local::now();
    let time = now.format("%Y-%m-%d %H:%M:%S").to_string();

    // Write to file
    let mut file = OpenOptions::new()
                    .write(true)
                    .append(true)
                    .open(file_name)
                    .unwrap();

    if let Err(e) = writeln!(file, "[{}]: {} from user: {}", time, request, username) {
        eprintln!("Couldn't write to file: {}", e);
    }
}

#[derive(Serialize, Deserialize)]
struct Person {
    name: String,
    age: u8,
    phones: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    #[serde(with = "json_string")]
    sms: Sms,
    uuid: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Sms {
    source: u64,
    destination: u64,
    content: String,
}

fn typed_example() -> Result<(Person), Error> {
    let data = r#"{
                    "name": "John Doe",
                    "age": 43,
                    "phones": [
                      "+44 1234567",
                      "+44 2345678"
                    ]
                  }"#;

    let p: Person = serde_json::from_str(data)?;

    Ok(p)
}

fn main() {

    match typed_example() {
        Ok(res) => println!("{}", res.name),
        Err(err) => println!("Error: {}", err),
    }


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
    create_log(&request, &username);

    // password - letters (lowercase AND uppercase) AND numbers, 8-50 characters
    // RegEx in Rust doesn't support look ahead assertion, so the RegEx would be very complicated
    if check_pass_requirement(&password) {
        println!("Password matches requirements");
    } else {
        println!("Password doesn't match requirements");
        return;
    }

    // // Hash the password
    match pbkdf2::pbkdf2_simple(&password, 1000) {
        Ok(_hash) => {
            hashed_password = _hash;
        },
        Err(_hash) => {
            hashed_password = "".to_string();
        }
    }

    // // Check the password hash against the password provided
    match pbkdf2::pbkdf2_check(&password, &hashed_password) {
        Ok(_x) => println!("Password matches password hash"),
        Err(_x) => println!("Password did not match password hash")
    }

    println!("{} {} {} \n{}",request, username, password, hashed_password);
}