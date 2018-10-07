use failure::{Backtrace, Context, Fail};
use std::convert::From;
use std::fmt::{self, Display};

use datatypes::auth::responses::AuthError;

/// The type of an internal error ([struct.Error.html])
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "unable to connect to database")]
    ConnectionError,
    #[fail(display = "a query failed to be executed")]
    QueryError,
    #[fail(display = "failed to start tarpc server")]
    ServerError,
    #[fail(display = "invalid username")]
    InvalidUsername,
    #[fail(display = "invalid password")]
    InvalidPassword,
    #[fail(display = "invalid token")]
    InvalidToken,
}

/// An internal error which can be used for debugging or error tracing
///
/// With this code set up, the error can be used by providing context method
/// from failure to apply your ErrorKind to errors in underlying libraries:
///
/// ```
/// # fn main() {
/// # let res = a().unwrap_err();
/// # let expt = Error { inner: ErrorKind::QueryError };
/// # assert_eq!(res, expt);
/// # }
/// # fn some_query() -> Result<u32, &'a str> {
/// # Err("invalid query")
/// # }
/// # fn a() -> Result<u32, Error> {
/// some_query().context(ErrorKind::QueryError)?;
/// # }
/// ```
///
/// Errors can also be directly throw as `ErrorKind` without an
/// underlying error when appropriate:
///
/// ```
/// # fn main() {
/// # let res = a().unwrap_err();
/// # let expt = Error { inner: ErrorKind::ConnectionError };
/// # assert_eq!(res, expt);
/// # }
/// # fn a() -> Result<u32, Error> {
/// Err(ErrorKind::ConnectionError)?
/// # }
/// ```
///
/// See [An Error and ErrorKind pair](https://boats.gitlab.io/failure/error-errorkind.html)
#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

impl Error {
    /// Get the type ([enum.ErrorKind.html]) of the error
    pub fn kind(&self) -> ErrorKind {
        *self.inner.get_context()
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner }
    }
}

impl Into<AuthError> for Error {
    fn into(self) -> AuthError {
        match self.kind() {
            ErrorKind::ConnectionError => AuthError::InternalServerError,
            ErrorKind::QueryError => AuthError::InternalServerError,
            ErrorKind::InvalidUsername => AuthError::InvalidUsername,
            ErrorKind::InvalidPassword => AuthError::InvalidPassword,
            ErrorKind::InvalidToken => AuthError::InvalidToken,
            ErrorKind::ServerError => AuthError::InternalServerError,
        }
    }
}