#[derive(Debug)]
pub enum Error {
    Connect(std::io::Error),
    Read(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Connect(err) => write!(f, "cannot connect: {}", err),
            Error::Read(err) => write!(f, "cannot read: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Connect(err) | Error::Read(err) => Some(err as &(dyn std::error::Error)),
        }
    }
}
