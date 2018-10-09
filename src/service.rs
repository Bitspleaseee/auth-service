use chrono::offset::Utc;
use chrono::DateTime;
use futures_cpupool::CpuFuture;
use futures_cpupool::CpuPool;
use pbkdf2::{pbkdf2_check, CheckError};
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::{Arc, RwLock};

use datatypes::auth::requests::*;
use datatypes::auth::responses::*;
use datatypes::content::requests::AddUserPayload;
use datatypes::valid::ids::UserId;
use datatypes::valid::token::Token;

use crate::db;
use crate::IntResult;

const PASS_PEPPER: &str = "4NqD&8Bh%d";
const HASH_PASS_CYCLES: u32 = 10000;

/// The auth server which will have the rpc services
#[derive(Clone)]
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

    // Pools
    pool: CpuPool,
    db_pool: db::DbPool,
}

impl AuthServer {
    /// Try to make a new server by creating a connection pool to the database
    pub fn try_new(database_url: &str) -> IntResult<Self> {
        let tokens = Arc::<RwLock<HashMap<Token, (UserId, Role, DateTime<Utc>)>>>::default();
        let db_pool = db::setup_connection_pool(database_url)?;

        Ok(AuthServer {
            tokens,
            pool: CpuPool::new_num_cpus(),
            db_pool,
        })
    }
}

service! {
    rpc authenticate(payload: AuthPayload) -> Token | AuthError;
    rpc deauthenticate(payload: Token) -> () | AuthError;
    rpc register(payload: RegisterUserPayload) -> AddUserPayload | AuthError;
    rpc get_user_role(payload: Token) -> Role | AuthError;
}

impl FutureService for AuthServer {
    type AuthenticateFut = CpuFuture<Token, AuthError>;
    type DeauthenticateFut = Result<(), AuthError>;
    type RegisterFut = CpuFuture<AddUserPayload, AuthError>;
    type GetUserRoleFut = Result<Role, AuthError>;

    fn get_user_role(&self, token: Token) -> Self::GetUserRoleFut {
        debug!("Received get role request for token: {:?}", &token);

        self.tokens
            .read()
            .map_err(|e| {
                error!("Unable to read 'tokens': {}", e);
                AuthError::InternalServerError
            })?.get(&token)
            .map(|(_, role, _)| { 
                trace!("Found token; role: {:?}", *role); 
                *role
            })
            .ok_or({
                trace!("No token found");
                AuthError::InvalidToken
            })
    }

    fn deauthenticate(&self, token: Token) -> Self::DeauthenticateFut {
        debug!("Received deauthenticate request for token: {:?}", &token);

        self.tokens
            .write()
            .map_err(|e| {
                error!("Unable to write to 'tokens': {}", e);
                AuthError::InternalServerError
            })?.remove(&token)
            .ok_or({
                AuthError::InvalidToken
            })?;

        trace!("Found and removed token");
        Ok(())
    }

    fn authenticate(&self, payload: AuthPayload) -> Self::AuthenticateFut {
        debug!("Received authentication request from: {}", &payload.username);

        let cloned_pool = self.db_pool.clone();
        let cloned_tokens = self.tokens.clone();

        let f = futures::lazy(move || {
            cloned_pool
                .get()
                .map_err(|e| {
                    error!("Unable to get a database connection from the pool: {}", e);
                    AuthError::InternalServerError
                }).and_then(|con| {
                    let AuthPayload {
                        username,
                        password: plain_password,
                    } = payload;

                    // Get hashed password from database for current username
                    let db::User {
                        password: hashed_password,
                        id: user_id,
                        ..
                    } = match db::fetch_user(&con, &username) {
                        Ok(v) => {
                            trace!("Found user");
                            v
                        },
                        Err(e) => {
                            trace!("User not found");
                            return Err(e.into())
                        },
                    };

                    // 'Pepper' the password
                    let pepper_pass = plain_password.into_inner() + &PASS_PEPPER;

                    // Check if the already stored hashed password matches the password
                    // that the user sent
                    match pbkdf2_check(&pepper_pass, &hashed_password) {
                        Ok(_) => {
                            trace!("Password matches");

                            let mut random_bytes = [0u8; 60];
                            thread_rng().fill(&mut random_bytes[..]);

                            let user_id = user_id.into();
                            let db::Role {
                                name: user_role, ..
                            } = match db::fetch_user_role(&con, user_id) {
                                Ok(v) => {
                                    trace!("Found user role");
                                    v
                                },
                                Err(e) => {
                                    trace!("Failed to find user role");
                                    return Err(e.into())
                                },
                            };

                            // Wrap in a empty scope, so that as soon as we're done writing
                            // to the 'HashMap', we'll drop the 'RwLockGuard' and hence make
                            // the 'HashMap' avaliable to other threads etc.
                            trace!("Generating token");
                            let token = Token::new(base64::encode(&random_bytes[..]));
                            let now = chrono::offset::Utc::now();
                            {

                                let token_clone = token.clone();
                                cloned_tokens
                                    .write()
                                    .map_err(|e| {
                                        error!("Unable to write to 'tokens': {}", e);
                                        AuthError::InternalServerError
                                    })?.insert(
                                        token_clone,
                                        (user_id.into(), user_role.as_str().into(), now),
                                    );
                            }
                            trace!("Returning token");
                            Ok(token)
                        }
                        // The password does NOT match, return `InvalidPassword`
                        Err(CheckError::HashMismatch) => {
                            trace!("Password does not match");
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
                })
        });
        self.pool.spawn(f)
    }

    fn register(&self, payload: RegisterUserPayload) -> Self::RegisterFut {
        debug!("Received register user request from: {}", &payload.username);

        let cloned_pool = self.db_pool.clone();

        let f = futures::lazy(move || {
            cloned_pool
                .get()
                .map_err(|e| {
                    error!("Unable to get a database connection from the pool: {}", e);
                    AuthError::InternalServerError
                }).and_then(|con| {
                    let RegisterUserPayload {
                        username,
                        password: plain_password,
                        email,
                    } = payload;

                    // Check if username is in DB
                    match db::fetch_user(&con, &username) {
                        Ok(_) => {
                            trace!("The username already exists");
                            return Err(AuthError::ExistingUser)
                        },
                        Err(_) => {}
                    };

                    // 'Pepper' the password
                    let pepper_pass = plain_password.into_inner() + &PASS_PEPPER;

                    // Hash the password of the user
                    trace!("Hashing password");
                    let hashed_password = pbkdf2::pbkdf2_simple(&pepper_pass, HASH_PASS_CYCLES)
                        .map_err(|e| {
                            error!("Unable to hash password, {}", e);
                            AuthError::InternalServerError
                        })?;

                    // Insert the user info into DB
                    trace!("Inserting user");
                    db::insert_user(
                        &con,
                        username.into_inner(),
                        email.into_inner(),
                        hashed_password,
                    ).map_err(|e| {
                        error!("Unable to insert user: {}", e);
                        e.into()
                    }).and_then(|user| {
                        let username = user.username;
                        let id = user.id;
                        username
                            .try_into()
                            .map_err(|_| AuthError::InvalidUsername)
                            .map(move |name| (id.into(), name))
                    }).map(|(id, username)| {trace!("Returning user payload"); AddUserPayload { id, username }})
                })
        });

        self.pool.spawn(f)
    }
}
