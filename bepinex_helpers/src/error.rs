use std::{error, fmt::Display, io};

#[derive(Debug)]
pub enum ErrorCode {
    Io(io::Error),
    InvalidGameType,
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::Io(e) => write!(f, "{}", e),
            ErrorCode::InvalidGameType => write!(f, "Invalid game type"),
        }
    }
}

#[derive(Debug)]
pub struct Error {
    code: ErrorCode,
}

impl Error {
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

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error {
            code: ErrorCode::Io(e),
        }
    }
}
