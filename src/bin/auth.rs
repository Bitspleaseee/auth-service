
extern crate diesel;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
use auth_service::*;
use self::models::*;
use diesel::prelude::*;

pub fn insert_user(conn: &MysqlConnection, user_name: &str, email: &str, password: &str, email_token: i32) -> Result<User> {

    use self::schema::users::dsl::*;
    use self::schema::roles::dsl::*;

    let new_user = NewUser {
        email: email.to_string(),
        username: user_name.to_string(),
        password: password.to_string(),
        email_token: email_token.to_string(),
    }

    diesel::insert(&new_user)
        .into(users)
        .execute(conn)
        .expect("Error creating new user");
    
    let fetched_user = fetch_user(conn,new_user.username)
    
    let new_role = NewRole {
        id: fetched_user.id,
        role:user
    }
    diesel::insert()
        .into(roles)
        .execute(conn)
        .expect("Failed updating role")

    return fetched_user
}


pub fn fetch_user(conn: &MysqlConnection, user_name: &str)-> IntResult<User> {
    use self::schema::users::dsl::*;

    let user = users::table
        .filter(username.eq(user_name))
        .first::<User>(conn)
        .expect("Error finding user");    
    return user    
}


pub fn update_ban(conn: &MysqlConnection, user_id: i32, banned_value: bool)->  bool {
    use self::schema::users::dsl::{id, users};
    let updated = diesel::update(users)
        .set(banned.eq(banned_value))
        .filter(id.eq(user_id))
        .execute(conn)
        .expect("Error finding user");

    updated > 0
}

pub fn update_verify(conn: &MysqlConnection, user_id: i32, verify_value: bool)->  bool {
    use self::schema::users::dsl::{id, users};
    let updated = diesel::update(users)
        .set(verified.eq(verified_value))
        .filter(id.eq(user_id))
        .execute(conn)
        .expect("Error finding user");
    updated > 0
        
}


pub fn update_email_token(conn: &MysqlConnection, user_id: i32, email_token_value: i32)->  bool {
    use self::schema::users::dsl::{id, users};
    let updated = diesel::update(users)
        .set(email_token.eq(email_token_value))
        .filter(id.eq(user_id))
        .execute(conn)
        .expect("Error finding user");

    updated > 0
}

pub fn update role(conn: &MysqlConnection, user_id: i32, new_role: String)-> bool {
    use schema::roles::dsl::*;
    let updated = diesel::update(roles)
        .set(name.eq(new_role))
        .filter(id.eq(user_id))
        .execute(cnn)
        .expect("Failed to update user role")

    updated > 0
}


pub fn fetch_user_role(conn: &MysqlConnection, user_id: i32) -> str {
    use self::schema::roles::dsl::{id, roles};

    let role = roles::table
        .filter(id.eq(user_id))
        .load::<String>(conn)?
        .expect("Error finding user");
    return role
}

