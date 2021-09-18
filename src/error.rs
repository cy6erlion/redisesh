use std::{fmt, str};

#[derive(Debug)]
pub enum Error {
    DBConnectionError,
    TokenCreationError,
    RedisResponseError,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::DBConnectionError => write!(f, "Error establishing connection with Redis"),
            Error::TokenCreationError => write!(f, "Error while generating random token"),
            Error::RedisResponseError => write!(f, "Redis response Error"),
        }
    }
}

impl From<redis::RedisError> for Error {
    fn from(err: redis::RedisError) -> Self {
        match err.kind() {
            redis::ErrorKind::ResponseError => Error::RedisResponseError,
            _ => Error::DBConnectionError,
        }
    }
}

impl From<str::Utf8Error> for Error {
    fn from(_: str::Utf8Error) -> Self {
        Error::TokenCreationError
    }
}
