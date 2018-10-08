use datatypes::auth::requests::*;
use datatypes::auth::responses::*;
use datatypes::content::requests::AddUserPayload;
use datatypes::payloads::*;
use datatypes::valid::ids::UserId;
use datatypes::valid::token::Token;

use pbkdf2::{pbkdf2_check, CheckError};
use rand::{thread_rng, Rng};

use db;

use chrono::offset::Utc;
use chrono::DateTime;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::convert::TryInto;

const PASS_PEPPER: &str = "4NqD&8Bh%d";
const HASH_PASS_CYCLES : u32 = 10000;

/// The auth server which will have the rpc services
#[derive(Clone, Default)]
pub struct AuthServer {
    /// An in-memory mapping between a Token and a UserId
    ///
    /// The hashmap also stores the time that the token was created. This can
    /// be used to remove stale tokens and force the creation of a new token on
    /// a given time interval.
    ///
    /// # Internal types
    ///
    /// - [`std::sync::Arc`] counts how many references there are to this
    /// object (threadsafely because its atomic).
    /// - [`std::sync::RwLock`] makes sure that either multiple references can
    /// read immutably OR one reference can mutate the 'HashMap'.
    tokens: Arc<RwLock<HashMap<Token, (UserId, Role, DateTime<Utc>)>>>,
}

service! {
    rpc authenticate(payload: AuthPayload) -> Token | AuthError;
    rpc deauthenticate(payload: TokenPayload<EmptyPayload>) -> () | AuthError;
    rpc register(payload: RegisterUserPayload) -> AddUserPayload | AuthError;
    rpc get_user_role(payload: TokenPayload<EmptyPayload>) -> Role | AuthError;
}

impl FutureService for AuthServer {
    type AuthenticateFut = Result<Token, AuthError>;
    type DeauthenticateFut = Result<(), AuthError>;
    type RegisterFut = Result<AddUserPayload, AuthError>;
    type GetUserRoleFut = Result<Role, AuthError>;

    fn get_user_role(&self, payload: TokenPayload<EmptyPayload>) -> Self::GetUserRoleFut {
        let (_, token) = payload.into_inner();

        self.tokens
            .read()
            .map_err(|e| {
                error!("Unable to read 'tokens': {}", e);
                AuthError::InternalServerError
            })?.get(&token)
            .map(|(_, role, _)| *role)
            .ok_or(AuthError::InvalidToken)
    }

    fn deauthenticate(&self, payload: TokenPayload<EmptyPayload>) -> Self::DeauthenticateFut {
        let (_, token) = payload.into_inner();

        // Remove the token from the HashMap
        {
            self.tokens
                .write()
                .map_err(|e| {
                    error!("Unable to write to 'tokens': {}", e);
                    AuthError::InternalServerError
                })?.remove(&token).ok_or(AuthError::InvalidToken)?;
        }

        Ok(())
    }

    fn authenticate(&self, payload: AuthPayload) -> Self::AuthenticateFut {
        let AuthPayload {
            username,
            password: plain_password,
        } = payload;

        info!("Received authentication request from '{}'", username);

        // Get hashed password from database for current username
        let db_connect = db::establish_connection();
        let db::User { password: hashed_password, id : user_id, .. } = match db::fetch_user(&db_connect, &username) {
            Ok(v) => v,
            Err(e) => return Err(e.into()),
        };
        
        // 'Pepper' the password
        let pepper_pass = plain_password.into_inner() + &PASS_PEPPER;

        // Check if the already stored hashed password matches the password
        // that the user sent
        match pbkdf2_check(&pepper_pass, &hashed_password) {
            Ok(_) => {
                info!("Password matches");

                let mut random_bytes = [0u8; 60];
                thread_rng().fill(&mut random_bytes[..]);

                let user_id = user_id.into();
                let token = Token::new(base64::encode(&random_bytes[..]));
                let now = chrono::offset::Utc::now();
                let db::Role {name: user_role, .. } = match db::fetch_user_role(&db_connect, user_id) {
                    Ok(v) => v,
                    Err(e) => return Err(e.into()),
                };

                // Wrap in a empty scope, so that as soon as we're done writing
                // to the 'HashMap', we'll drop the 'RwLockGuard' and hence make
                // the 'HashMap' avaliable to other threads etc.
                {
                    let token_clone = token.clone();
                    self.tokens
                        .write()
                        .map_err(|e| {
                            error!("Unable to write to 'tokens': {}", e);
                            AuthError::InternalServerError
                        })?.insert(token_clone, (user_id.into(), user_role.as_str().into(), now));
                }
                Ok(token)
            }
            // The password does NOT match, return `InvalidPassword`
            Err(CheckError::HashMismatch) => {
                info!("Password does not match");
                Err(AuthError::InvalidPassword)
            }
            // TODO handle the situation where the internally stored password
            // is badly formatted
            //
            // The internal 'hashed_password' does not have the correct
            // format. This is probably a result of corruption. This
            // will hopefully be a rare occurence. Perhaps reset password?
            Err(CheckError::InvalidFormat) => {
                error!("Hashed password has invalid format");
                Err(AuthError::InternalServerError)
            }
        }
    }

    fn register(&self, payload: RegisterUserPayload) -> Self::RegisterFut {
        let RegisterUserPayload {
            username,
            password: plain_password,
            email,
        } = payload;

        info!("Received register request from '{}'", username);

        // Check if username is in DB
        let db_connect = db::establish_connection();
        match db::fetch_user(&db_connect, &username) {
            Ok(_) => return Err(AuthError::InvalidUsername),
            Err(_) => {},
        };

        // 'Pepper' the password
        let pepper_pass = plain_password.into_inner() + &PASS_PEPPER;

        // Hash the password of the user
        let hashed_password = pbkdf2::pbkdf2_simple(&pepper_pass, HASH_PASS_CYCLES).map_err(|e| {
            error!("Unable to hash password, {}", e);
            AuthError::InternalServerError
        })?;

        
        // Insert the user info into DB
        db::insert_user(&db_connect, username.into_inner(), email.into_inner(), hashed_password)
            .map_err(|e| {
                error!("Unable to insert user: {}", e);
                e.into()
            })
            .and_then(|user| {
                let username = user.username;
                let id = user.id;
                username
                    .try_into()
                    .map_err(|_| AuthError::InvalidUsername)
                    .map(move |name| (id.into(), name))})
            .map(|(id, username)| AddUserPayload{id, username})

    }

    /* implement the other services and their return type here */
}
