

use std::env;
use dotenv::dotenv;

use diesel::prelude::*;
use crate::schema::*;
use crate::{IntResult,IntErrorKind};
use failure::ResultExt;
use diesel::result::Error;
/*
Connects to database to URL set in .env
*/
pub fn establish_connection() -> MysqlConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}


#[derive(Queryable, PartialEq, Debug)]
pub struct User {
    pub id: u32,
    pub email: String,
    pub username: String,
    pub password: String,
    pub banned: bool,
    pub verified: bool,
    pub email_token: Option <String>,
}


#[derive(Queryable, PartialEq, Debug)]
pub struct Role {
    pub id: u32,
    pub name: String,

}

#[derive(Debug, PartialEq, Insertable)]
#[table_name="users"]
pub struct NewUser {
    pub email: String,
    pub username: String,
    pub password: String,
}


#[derive(Debug,PartialEq, Insertable)]
#[table_name="roles"]
pub struct NewRole {
    pub id: u32,
    pub name: String,
}

/*
Creates a user based on username, email and hashed password.
Updates the newly created user's role.
Returns newly created user

*/

pub fn insert_user(conn: &MysqlConnection, user_name: String, new_email: String, new_password: String) -> IntResult<User>  {
    use schema::roles::dsl::*;
    use schema::users::dsl::*;
    let new_user = NewUser {
        email: new_email,
        username: user_name.clone(),
        password: new_password,
    };

    diesel::insert_into(users)
        .values(&new_user)
        .execute(conn)
        .context(IntErrorKind::QueryError)
        .map_err(|e| {
            error!("Unable to insert user: {}", e);
            e
        })?;
    let fetched_user = fetch_user(conn, &user_name)?; 

    let new_role = NewRole {
        id: fetched_user.id,
        name: "user".to_string(),
    };

    diesel::insert_into(roles)
        .values(&new_role)
        .execute(conn)
        .context(IntErrorKind::QueryError)
        .map_err(|e| {
            error!("Unable to insert user role: {}", e);
            e
    })?;

    Ok(fetched_user)

}


/*
Returns user based on user username
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



#[test]
fn test_insert_user() {
    let mut test_user = User {
        id: 0,
        email: "test_email".to_string(),
        username: "test_username".to_string(),
        password: "test_password".to_string(),
        banned: false,
        verified: false,
        email_token: Option::None,
    };

    let new_user = NewUser {
        username: "test_username".to_string(),
        email: "test_email".to_string(),
        password: "test_password".to_string(),
    };

    
    let conn = establish_connection();
    &conn.transaction::<(), _, _>(|| {
        let userv = insert_user(&conn, new_user.username,new_user.email,new_user.password);
        let user = userv.unwrap();
        test_user.id += user.id;
        let test_role = Role {
            id: user.id,
            name: "user".to_string(),
        };
        let role = fetch_user_role(&conn, test_role.id);

        assert_eq!(test_user,user);
        assert_eq!(test_role,role.unwrap());

        Err(Error::RollbackTransaction)
    });

}


#[test]
fn test_insert_user_fail() {

    let new_user = NewUser {
        username: "REEEEEEEEEEEEEEEEEEEEEEEEEEEEEEE".to_string(),
        email: "email_test1".to_string(),
        password: "password1".to_string()
    };

    let conn = establish_connection();
    &conn.transaction::<(), _, _>(|| {
        let user = insert_user(&conn, new_user.username, new_user.email, new_user.password);
        assert!(user.is_err());
        Err(Error::RollbackTransaction)
    });    
 }


#[test]
fn test_update_functions() {

    let mut test_user = User {
        id: 0,
        email: "email1".to_string(),
        username: "username1".to_string(),
        password: "password1".to_string(),
        banned: true,
        verified: true,
        email_token: Option::Some("123456789".to_string()),
    };

    let new_user = NewUser {
        username: "username1".to_string(),
        email: "email1".to_string(),
        password: "password1".to_string()
    };
    let conn = establish_connection();
    &conn.transaction::<(), _, _>(|| {
        let userv = insert_user(&conn, new_user.username, new_user.email, new_user.password);
        let user = userv.unwrap();
        test_user.id += user.id;
        let test_role = Role {
            id: test_user.id,
            name: "moderator".to_string(),
        };
        assert_eq!(true, update_role(&conn, user.id, "moderator".to_string()).unwrap());
        let new_user_role = fetch_user_role(&conn, user.id);
        assert_eq!(test_role, new_user_role.unwrap());

        assert_eq!(true, update_ban(&conn, user.id, true).unwrap());
        assert_eq!(true, update_email_token(&conn, user.id, "123456789".to_string()).unwrap());
        assert_eq!(true, update_verify(&conn, user.id, true).unwrap());

        let userv = fetch_user(&conn, &test_user.username);
        assert_eq!(test_user,userv.unwrap());
        Err(Error::RollbackTransaction)
    });
}





