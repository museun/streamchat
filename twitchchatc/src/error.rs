use std::fmt;

#[derive(Debug)]
pub enum Error {
    CannotConnect,
    CannotRead,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::CannotConnect => write!(f, "cannot connect"),
            Error::CannotRead => write!(f, "cannot read"),
        }
    }
}
