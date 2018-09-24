extern crate serde_json;
use serde_json::{Value, Error};

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


fn untyped_example() -> Result<(), Error> {
    // Some JSON input data as a &str. Maybe this comes from the user.
    let data = r#"{
                    "name": "John Doe",
                    "age": 43,
                    "phones": [
                      "+44 1234567",
                      "+44 2345678"
                    ]
                  }"#;

    // Parse the string of data into serde_json::Value.
    let v: Value = serde_json::from_str(data)?;

    // Access parts of the data by indexing with square brackets.
    println!("Please call {} at the number {}", v["name"], v["phones"][1]);

    Ok(())
}

fn main() {
    untyped_example();
}
