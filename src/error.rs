use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    DivisionByZero,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DivisionByZero => write!(f, "Attempted to divide by zero"),
        }
    }
}
