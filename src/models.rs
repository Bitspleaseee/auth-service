

pub struct user {
    pub id: i32,
    pub email: String,
    pub username: String,
    pub password: String,
    pub banned: bool,
    pub verified: bool,
    pub email_token: i32,
}

pub struct role {
    pub id: i32,
    pub name: String,
}