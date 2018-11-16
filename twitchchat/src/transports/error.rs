use std::fmt;

#[derive(Debug)]
pub enum Error {
    Full,
    Disconnected,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Full => write!(f, "buffer is full"),
            Error::Disconnected => write!(f, "disconnected"),
        }
    }
}
