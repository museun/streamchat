use log::{error, info, trace, warn};

macro_rules! import {
    ($($arg:ident),*) => {
        $(
            pub mod $arg;
            pub use self::$arg::*;
        )*
    };
}

import!(
    command,    //
    conn,       //
    error,      //
    ircmessage, //
    mockconn,   //
    options,    //
    server,     //
    tcpconn,    //
    queue,      //
    transports  //
);

pub(crate) use twitchchat::prelude::*;

pub(crate) fn make_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    ts.as_secs() * 1000 + u64::from(ts.subsec_nanos()) / 1_000_000
}
