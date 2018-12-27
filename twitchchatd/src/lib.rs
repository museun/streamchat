use log::{error, info, trace, warn};
use std::fmt;

macro_rules! import {
    ($($arg:ident),*) => {
        $(
            pub mod $arg;
            pub use self::$arg::*;
        )*
    };
}

import!(
    ircmessage, //
    mockconn,   //
    options,    //
    server,     //
    tcpconn,    //
    queue,      //
    transports, //
    custom      //
);

pub(crate) use twitchchat::prelude::*;

#[derive(Debug, PartialEq)]
pub enum Maybe {
    Just(String),
    Nothing,
}

pub trait Conn {
    fn try_read(&mut self) -> Option<Maybe>;
    fn write(&mut self, data: &str) -> Result<usize, Error>;
}

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

pub(crate) fn make_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    ts.as_secs() * 1000 + u64::from(ts.subsec_nanos()) / 1_000_000
}
