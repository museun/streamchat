use twitchchatd::*;

use log::{error, info};
use std::env;
use std::io::BufRead;

fn main() {
    let (name, args) = {
        let mut args = env::args();
        (args.next().unwrap(), args.collect::<Vec<_>>())
    };
    let options = Options::parse(&name, &args);
    init_logger(&options.log_level, options.use_colors);

    let token = match env::var("TWITCH_CHAT_OAUTH_TOKEN") {
        Ok(token) => token,
        Err(..) => {
            error!("TWITCH_CHAT_OAUTH_TOKEN must be set to oauth:token");
            std::process::exit(1);
        }
    };

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
        vec![Box::new(Socket::start(&options.addr, options.limit))],
    );

    info!("starting server");
    server.run();
    info!("exiting");
}

fn init_logger(log_level: &Level, colors: bool) {
    use simplelog::*;

    let filter = match log_level {
        self::Level::Off => LevelFilter::Off,
        self::Level::Trace => LevelFilter::Trace,
        self::Level::Debug => LevelFilter::Debug,
        self::Level::Info => LevelFilter::Info,
        self::Level::Warn => LevelFilter::Warn,
        self::Level::Error => LevelFilter::Error,
    };

    let config = Config::default();
    if colors {
        TermLogger::init(filter, config).expect("enable logging");
    } else {
        SimpleLogger::init(filter, config).expect("enable logging");
    }
}
