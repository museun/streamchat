use twitchchatd::*;

use log::{error, info};
use std::env;

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

    let addr = "irc.chat.twitch.tv:6667";

    let conn = match TcpConn::connect(&addr, &token, &options.channel, &options.nick) {
        Ok(conn) => conn,
        Err(_err) => {
            error!("cannot connect");
            std::process::exit(1)
        }
    };

    let mut server = Server::new(
        conn,
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
