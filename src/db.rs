

use std::env;
use dotenv::dotenv;

use diesel::prelude::*;
use crate::schema::*;
use crate::{IntResult,IntErrorKind};
use failure::ResultExt;
/*
Connects to database to URL set in .env
*/
pub fn establish_connection() -> MysqlConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}


#[derive(Queryable)]
pub struct User {
    pub id: u32,
    pub email: String,
    pub username: String,
    pub password: String,
    pub banned: bool,
    pub verified: bool,
    pub email_token: Option <String>,
}


#[derive(Queryable)]
pub struct Role {
    pub id: u32,
    pub name: String,

}

#[derive(Debug, Insertable)]
#[table_name="users"]
pub struct NewUser {
    pub email: String,
    pub username: String,
    pub password: String,
}


#[derive(Debug, Insertable)]
#[table_name="roles"]
pub struct NewRole {
    pub id: u32,
    pub name: String,
}

/*
TO DO:
Needs to return a vector which i dont know how to do.

*/
/*
pub fn insert_user(conn: &MysqlConnection, user_name: &str, new_email: &str, new_password: &str, email_token: &str) -> IntResult<User>  {
    use crate::schema::dsl::*;
    use crate::schema::roles::dsl::*;
    use crate::schema::users::table as users;

    let new_user = NewUser {
        email: new_email.to_string(),
        username: user_name.to_string(),
        password: new_password.to_string(),
    };

    let inserted = 
    diesel::insert_into(users)
        .values(&new_user)
        .execute(conn)
        .context()

    if inserted == 1 {
        let fetched_user = users
                        .filter(username.eq(user_name))
                        .first::<User>(conn)
                        .unwrap();
    
        let new_role = NewRole {
            id: fetched_user.id,
            name: "user".to_string(),
        };

        diesel::insert_into(roles)
            .values(&new_role)
            .execute(conn)
            .expect("Failed updating role");


    }
    else {
        // ????
    }

    return fetched_user
}
*/

/*
Needs to return vector
*/

pub fn fetch_user(conn: &MysqlConnection, new_username: &str)-> IntResult<User> {
    use crate::schema::users::dsl::*;
    
    users
        .filter(username.eq(new_username))
        .first(conn)
        .optional()
        .context(IntErrorKind::QueryError)?
        .ok_or(IntErrorKind::InvalidUsername)
        .map_err(|e| {
            error!("Unable to fetch user: {}", e);
            e.into()
        })

}

/*
Updates banned status of a user based on user id.
Returns true if updated, false if not.
*/
pub fn update_ban(conn: &MysqlConnection, user_id: u32, banned_value: bool)->  IntResult<bool> {
    use crate::schema::users::dsl::*;
    let updated = diesel::update(users)
        .set(banned.eq(banned_value))
        .filter(id.eq(user_id))
        .execute(conn)
        .context(IntErrorKind::QueryError)
        .map_err(|e| {
            error!("Failed to update banned status: {}", e);
            e
        })?;

    Ok(updated > 0)
}

/*
Updates verified status of a user based on user id.
Returns true if updated, false if not.
*/
pub fn update_verify(conn: &MysqlConnection, user_id: u32, verify_value: bool)->  IntResult<bool> {
    use crate::schema::users::dsl::*;
    let updated = diesel::update(users)
        .filter(id.eq(user_id))
        .set(verified.eq(verify_value))
        .execute(conn)
        .context(IntErrorKind::QueryError)
        .map_err(|e| {
            error!("Failed to update verified status: {}", e);
            e
        })?;

    Ok(updated > 0)    
}

/*
Updates email_token of a user based on user id
Returns true if updated, false if not.
*/
pub fn update_email_token(conn: &MysqlConnection, user_id: u32, email_token_value: String)->  IntResult<bool> {
    use crate::schema::users::dsl::*;
    let updated = diesel::update(users)
        .set(email_token.eq(email_token_value))
        .filter(id.eq(user_id))
        .execute(conn)
        .context(IntErrorKind::QueryError)
        .map_err(|e| {
            error!("Failed to update email token: {}", e);
            e
        })?;

    Ok(updated > 0)
}

/*
Updates role status of a user based on user id.
Returns true if updated, false if not.
*/
pub fn update_role(conn: &MysqlConnection, user_id: u32, new_role: String)-> IntResult<bool> {
    use schema::roles::dsl::*;
    let updated = diesel::update(roles)
        .set(name.eq(new_role))
        .filter(id.eq(user_id))
        .execute(conn)
        .context(IntErrorKind::QueryError)
        .map_err(|e| {
            error!("Failed to update user role: {}", e);
            e
        } )?;

    Ok(updated > 0)
}

/*
Fetches a users role based on user id
Returns string
*/


pub fn fetch_user_role(conn: &MysqlConnection, user_id: u32) -> IntResult<Role> {
    use schema::roles::dsl::*;
    roles
        .filter(id.eq(user_id))
        .first::<Role>(conn)
        .optional()
        .context(IntErrorKind::QueryError)?
        .ok_or(IntErrorKind::ServerError)
        .map_err(|e| {
            error!("Failed to fetch user role: {}", e);
            e.into()
        } )
}


