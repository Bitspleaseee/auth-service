// extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

#[derive(Serialize, Deserialize)]
enum UserRole {
    Guest,
    User,
    Moderator,
    Administrator,
}

#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
    email: String,
    username: String,
    password: String,
    salt: String,
    role: UserRole,
    banned: bool,
    verified: bool,
    email_token: String,
}

fn get_register_info(json_data: String) -> (String, String, String) {

    let u: User = serde_json::from_str(json_data);

    // println!("{} {} {}", u.email, u.username, u.password);
    (u.email, u.username, u.password)
}

// fn untyped_example() -> Result<(), Error> {
//     // Some JSON input data as a &str. Maybe this comes from the user.
//     let data = r#"{
//                     "name": "John Doe",
//                     "age": 43,
//                     "phones": [
//                       "+44 1234567",
//                       "+44 2345678"
//                     ]
//                   }"#;

//     // Parse the string of data into serde_json::Value.
//     let v: Value = serde_json::from_str(data)?;

//     // Access parts of the data by indexing with square brackets.
//     println!("Please call {} at the number {}", v["name"], v["phones"][1]);

//     Ok(())
// }

fn main() {
    //The JSON data received from another module
    let json_data = r#"{
                        "email": "test@somedomain.com",
                        "username": "someUsername",
                        "password": "someuserpass"
                      }"#;

    get_register_info(json_data);
}
