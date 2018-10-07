use datatypes::auth::requests::{AuthPayload, RegisterUserPayload};
use datatypes::auth::responses::{AuthError, AuthSuccess};
use datatypes::payloads::*;
use datatypes::valid::ids::UserId;
use datatypes::valid::token::Token;

use pbkdf2::{pbkdf2_check, CheckError};

use chrono::offset::Utc;
use chrono::DateTime;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

const PASS_PEPPER: &str = "4NqD&8Bh%d";

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug, Serialize, Deserialize)]
pub enum Role {
    Admin,
    Moderator,
    User,
}

impl<'a> From<&'a str> for Role {
    fn from(s: &'a str) -> Self {
        match s {
            "admin" => Role::Admin,
            "moderator" => Role::Moderator,
            "user" => Role::User,
            _ => Role::User,
        }
    }
}

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
    rpc register(payload: RegisterUserPayload) -> () | AuthError;
    rpc get_user_role(payload: TokenPayload<EmptyPayload>) -> Role | AuthError;

    /* more services should be defined (deauthenticate etc) */
}

impl FutureService for AuthServer {
    type AuthenticateFut = Result<Token, AuthError>;
    type DeauthenticateFut = Result<(), AuthError>;
    type RegisterFut = Result<(), AuthError>;
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
                })?.remove(&token);
        };

        Ok(())
    }

    fn authenticate(&self, payload: AuthPayload) -> Self::AuthenticateFut {
        let AuthPayload {
            username,
            password: plain_password,
        } = payload;

        info!("Received authentication request from '{}'", username);

        // Get hashed password from database for current username
        let hashed_password = /* TODO get hashed password from database */ "";

        // Check if the already stored hashed password matches the password
        // that the user sent
        match pbkdf2_check(&plain_password, &hashed_password) {
            Ok(_) => {
                info!("Password matches");

                let user_id = /* TODO get user_id from database */ 1.into();
                let token = Token::new(/* TODO generate random string? */ "random");
                let now = chrono::offset::Utc::now();
                let role = /* Get from DB*/ "admin";

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
                        })?.insert(token_clone, (user_id, role.into(), now))
                };
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

        // 'Pepper' the password
        let pepper_pass = plain_password.into_inner() + &PASS_PEPPER;

        // Hash the password of the user
        let hashed_password = pbkdf2::pbkdf2_simple(&pepper_pass, 10000).map_err(|e| {
            error!("Unable to hash password, {}", e);
            AuthError::InternalServerError
        })?;

        // Insert the user info into DB
        Ok(())
    }

    /* implement the other services and their return type here */
}
