use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Error {
    CannotWrite,
    CannotConnect,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::CannotWrite => write!(f, "cannot write"),
            Error::CannotConnect => write!(f, "cannot connect"),
        }
    }
}
