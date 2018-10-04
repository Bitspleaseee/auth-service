use schema::{users,roles};

#[derive(Queryable)]
pub struct user {
    pub id: i32,
    pub email: String,
    pub username: String,
    pub password: String,
    pub banned: bool,
    pub verified: bool,
    pub email_token: i32,
}

#[derive(Debug, Insertable)]
#[table_name="users"]
pub struct NewUser {
    pub email: String,
    pub username: String,
    pub password: String,
    pub email_token: i32, 
}

#[derive(Queryable)]
pub struct role {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Insertable)]
#[table_name="roles"]
pub struct NewRole {
    pub id: i32,
    pub name: String,
}