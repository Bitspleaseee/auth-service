extern crate serde_json;

use serde_json::{Value,Error};

// enum UserRole {
//     Guest,
//     User,
//     Moderator,
//     Administrator,
// }

// struct User {
//     id: u32,
//     email: String,
//     username: String,
//     password: String,
//     salt: String,
//     role: UserRole,
//     banned: bool,
//     verified: bool,
//     email_token: String,
// }

fn parse_json_data(s: &str) -> Result<Value, Error> {
    // Parse the string of data into serde_json::Value.
    let v: Value = serde_json::from_str(&s)?;
    Ok(v)
}

fn main() {

    // Static JSON data, for now...
    let x = r#"{
                    "email": "test@somedomain.com",
                    "username": "someUsername",
                    "password": "someuserpass"
                  }"#;

    // serde_json::from_str seems to return a Result type
    // This is the only way I could get Result<T, E>
    // to work while returning the parsed JSON data to main()
    match parse_json_data(x) {
        Ok(v) => println!("{}", v["email"]),
        Err(v) => println!("Error {}", v),
    }
}
