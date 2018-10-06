use datatypes::auth::requests::AuthPayload;
use datatypes::auth::responses::{AuthError, AuthSuccess};
use datatypes::valid::ids::UserId;
use datatypes::valid::token::Token;

use pbkdf2::{pbkdf2_check, CheckError};

use chrono::offset::Utc;
use chrono::DateTime;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

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
    tokens: Arc<RwLock<HashMap<Token, (UserId, DateTime<Utc>)>>>,
}

service! {
    rpc authenticate(payload: AuthPayload) -> Token | AuthError;
    /* more services should be defined (deauthenticate etc) */
}

impl FutureService for AuthServer {
    type AuthenticateFut = Result<Token, AuthError>;

    fn authenticate(&self, payload: AuthPayload) -> Self::AuthenticateFut {
        let AuthPayload {
            username,
            password: plain_password,
        } = payload;

        info!("Recived authentication request from '{}'", username);

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
                        })?.insert(token_clone, (user_id, now))
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

    /* implement the other services and their return type here */
}
