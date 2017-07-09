use diesel;
use bcrypt;
use std::io;
use std::result;
use std::num;
use r2d2::GetTimeout;
use jwt::errors;
use jwt::errors::ErrorKind;
use std::fmt;

pub enum Error {
    GetDbError,
    NoResultError,
    DieselError,
    PasswordHashError,
    InvalidPasswordError,
    InvalidUsernameError,
    PasswordMatchError,
    UserNotVerifiedError,
    InvalidWebtokenError,
    ExpiredWebtokenError,
    ParseError,
    IOError,
}

pub type Result<T> = result::Result<T, Error>;

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match *self {
            Error::GetDbError => "Timed out while waiting for database".to_string(),
            Error::NoResultError => "Could not find requested resource".to_string(),
            Error::DieselError => "Invalid database interaction".to_string(),
            Error::PasswordHashError => "Could not hash password".to_string(),
            Error::InvalidPasswordError => "Password did not meet requirements".to_string(),
            Error::InvalidUsernameError => "Username did not meet requirements".to_string(),
            Error::PasswordMatchError => "Passwords do not match".to_string(),
            Error::UserNotVerifiedError => "User is not verified".to_string(),
            Error::InvalidWebtokenError => "Webtoken is invalid".to_string(),
            Error::ExpiredWebtokenError => "Webtoken has expired".to_string(),
            Error::ParseError => "Could not parse data from string".to_string(),
            Error::IOError => "Something went wrong".to_string(),
        }
    }
}

impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Error {
        match e {
            diesel::result::Error::NotFound => Error::NoResultError,
            _ => Error::DieselError,
        }
    }
}

impl From<GetTimeout> for Error {
    fn from(_: GetTimeout) -> Error {
        Error::GetDbError
    }
}

impl From<bcrypt::BcryptError> for Error {
    fn from(_: bcrypt::BcryptError) -> Error {
        Error::PasswordHashError
    }
}

impl From<errors::Error> for Error {
    fn from(e: errors::Error) -> Error {
        match *e.kind() {
            ErrorKind::ExpiredSignature => Error::ExpiredWebtokenError,
            _ => Error::InvalidWebtokenError,
        }
    }
}

impl From<num::ParseIntError> for Error {
    fn from(_: num::ParseIntError) -> Error {
        Error::ParseError
    }
}

impl From<io::Error> for Error {
    fn from(_: io::Error) -> Error {
        Error::IOError
    }
}
