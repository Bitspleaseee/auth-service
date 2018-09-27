extern crate serde_json;
extern crate pbkdf2;
extern crate regex;

use regex::Regex;
use serde_json::{Value,Error};

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
fn check_password(input: &str) -> bool {
    
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

fn main() {

    // Some variables
    let request : String;
    let username : String;
    let password : String;
    let hashed_password : String;

    // This is easier to work with
    let x = r#"{
        "type": "AUTHENTICATE",
        "username": "james",
        "password": "testPass321"
    }"#;

    // serde_json::from_str seems to return a Result type
    // This is the only way I could get Result<T, E>
    // to work while returning the parsed JSON data to main()
    match parse_json_data(x) {
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
    
    // Regular Expression magic
    // username - letters (lowercase AND/OR uppercase) OR numbers, 5-20 characters
    let re = Regex::new(r"^[a-zA-Z0-9]{5,20}$").unwrap();
    if re.is_match(&username) {
        println!("Username matches the requirements");
    } else {
        println!("Username does not match the requirements");
        return;
    }

    // password - letters (lowercase AND uppercase) AND numbers, 8-50 characters
    // RegEx in Rust doesn't support look ahead assertion, so the RegEx would be very complicated
    if check_password(&password) {
        println!("It is 8-50 characters");
    } else {
        return;
    }

    // Hash the password
    match pbkdf2::pbkdf2_simple(&password, 1000) {
        Ok(_hash) => {
            hashed_password = _hash;
        },
        Err(_hash) => {
            hashed_password = "".to_string();
        }
    }

    // Check the password hash against the password provided
    match pbkdf2::pbkdf2_check(&password, &hashed_password) {
        Ok(_x) => println!("Password matches password hash"),
        Err(_x) => println!("Password did not match password hash")
    }

    println!("{} {} {} \n{}",request, username, password, hashed_password);
}