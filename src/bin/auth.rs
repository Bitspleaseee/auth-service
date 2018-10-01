extern crate chrono;
extern crate diesel;
extern crate serde_derive;
extern crate serde_json;
use auth_service::*;
use self::models::*;
use diesel::prelude::*;

pub fn insert_user(conn: &MysqlConnection, user_name: &str, email: &str, password: &str, email_token: i32) -> Result<User> {

    use self::schema::users::dsl::{id, users};

    let new_user = NewUser {
        email: email,
        username: user_name,
        password: password,
        email_token: email_token,
    }

    diesel::insert_into(users)
        .values(&new_user)
        .execute(conn)
        .expect("Error creating new user");
}


pub fn fetch_user(conn: &MysqlConnection, user_name: &str)-> Result<User> {
    use self::schema::users::dsl::{id, users};

    let user = users::table
        .filter(username.eq(user_name))
        .first::<User>(conn)
        .expect("Error finding user");        
}


pub fn update_ban(conn: &MysqlConnection, user_id: i32, banned_value: bool)->  bool {
    use self::schema::users::dsl::{id, users};
    let updated = diesel::update(users)
        .set(banned.eq(banned_value))
        .filter(id.eq(user_id))
        .execute(conn)
        .expect("Error finding user");

    if (updated != 0) {
        updated = true
}

pub fn update_verify(conn: &MysqlConnection, user_id: i32, verify_value: bool)->  bool {
    use self::schema::users::dsl::{id, users};
    let updated = diesel::update(users)
        .set(verified.eq(verified_value))
        .filter(id.eq(user_id))
        .execute(conn)
        .expect("Error finding user");

    if (updated != 0) {
        updated = true
}


pub fn update_email_token(conn: &MysqlConnection, user_id: i32, email_token__value: i32)->  bool {
    use self::schema::users::dsl::{id, users};
    let updated = diesel::update(users)
        .set(email_token.eq(email_token__value))
        .filter(id.eq(user_id))
        .execute(conn)
        .expect("Error finding user");

    if (updated != 0) {
        updated = true
}


pub fn fetch_user_role(conn: &MysqlConnection, user_id: i32) -> str {
    use self::schema::roles::dsl::{id, roles};

    let role = roles::table
        .filter(id.eq(user_id))
        .load::<String>(conn)?
        .expect("Error finding user");
}

