#![feature(slice_patterns)]
use twitchchat::prelude::*;
use twitchchat::transports::{Socket, Transport};

use std::env;
use std::io::BufRead;

mod command;
mod conn;
mod error;
mod ircmessage;
mod mockconn;
mod options;
mod server;
mod tags;
mod tcpconn;

pub use self::{
    command::*,
    conn::*,
    error::*,
    ircmessage::*,
    mockconn::*,
    options::*,
    server::*,
    tags::*,
    tcpconn::*,
};

fn main() {
    let token = match env::var("TWITCH_CHAT_OAUTH_TOKEN") {
        Ok(token) => token,
        Err(..) => {
            eprintln!("TWITCH_CHAT_OAUTH_TOKEN must be set to oauth:token");
            std::process::exit(1);
        }
    };

    let (name, args) = {
        let mut args = env::args();
        (args.next().unwrap(), args.collect::<Vec<_>>())
    };
    let options = Options::parse(&name, &args);

    let mut server = Server::new(
        match (options.file, options.stdin) {
            (None, None) => {
                let addr = "irc.chat.twitch.tv:6667";
                Box::new(
                    TcpConn::connect(&addr, &token, &options.channel, &options.nick)
                        .expect("listen"),
                )
            }
            (Some(fd), None) => Box::new(MockConn::new(fd.lines())),
            (None, Some(fd)) => Box::new(MockConn::new(fd.lines())),
            _ => unreachable!(),
        },
        vec![
            Box::new(Socket::start(&options.addr, options.limit)), //
        ],
    );

    server.run();
}
