use std::{error, fmt::Display, io};

use reqwest::StatusCode;

#[derive(Debug)]
pub enum ErrorCode {
    Http(StatusCode),
    Reqwest(reqwest::Error),
    Io(io::Error),
    ZipError(zip::result::ZipError),
    InvalidGameType,
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::Reqwest(e) => write!(f, "{}", e),
            ErrorCode::Io(e) => write!(f, "{}", e),
            ErrorCode::Http(code) => write!(f, "HTTP Response code: {}", code),
            ErrorCode::ZipError(e) => write!(f, "{}", e),
            ErrorCode::InvalidGameType => write!(f, "Invalid game type"),
        }
    }
}

#[derive(Debug)]
pub struct Error {
    code: ErrorCode,
}

impl Error {
    pub fn http(code: StatusCode) -> Self {
        Error {
            code: ErrorCode::Http(code),
        }
    }

    pub fn zip_error(e: zip::result::ZipError) -> Self {
        Error {
            code: ErrorCode::ZipError(e),
        }
    }

    pub fn invalid_game_type() -> Self {
        Error {
            code: ErrorCode::InvalidGameType,
        }
    }
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error {
            code: ErrorCode::Reqwest(e),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error {
            code: ErrorCode::Io(e),
        }
    }
}

impl From<zip::result::ZipError> for Error {
    fn from(e: zip::result::ZipError) -> Self {
        Error {
            code: ErrorCode::ZipError(e),
        }
    }
}
