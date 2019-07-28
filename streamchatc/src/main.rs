mod layout;
mod window;
use window::Window;

mod args;
use args::{Args, Config};

use configurable::Configurable as _;
use crossbeam_channel as channel;
use gumdrop::Options as _;
use std::io::{BufRead, BufReader};
use streamchat::connection as conn;
use yansi::{Color, Paint};

fn main() {
    let args = Args::parse_args_default_or_exit();
    if args.print_config {
        eprintln!("{}", Config::path().unwrap().display());
        std::process::exit(0);
    }

    let config = if !*args.config {
        log::trace!("creating config from args");
        Config::create_config_from_args(&args)
    } else {
        log::trace!("loading config from file");
        Config::load_config_and_override(&args)
    };

    let color = std::env::var("NO_COLOR").is_err();
    if cfg!(windows) && !Paint::enable_windows_ascii() || !color {
        Paint::disable();
    }

    let client = match if args.standalone {
        log::trace!("using standalone client");
        Client::standalone(&config)
    } else {
        log::trace!("connecting to remote server");
        Client::connect_to_server(&config)
    } {
        Ok(client) => client,
        Err(err) => {
            eprintln!("cannot connect: {}", err);
            std::process::exit(1)
        }
    };

    log::trace!("starting window");
    Window::run(config, client.recv.clone());
    log::trace!("window done drawing");

    if let Err(err) = client.wait_for_end() {
        eprintln!("error: {}", err);
        // is this really an error
    }
}

struct Client {
    handle: std::thread::JoinHandle<Result<(), conn::Error>>,
    recv: channel::Receiver<streamchat::Message>,
}

impl Client {
    fn standalone(config: &Config) -> Result<Self, conn::Error> {
        let conn = conn::connect_to_twitch(&config.nick, &config.token, &config.channel)?;

        let (tx, rx) = channel::unbounded();
        let handle = std::thread::spawn(move || conn::read_until_end(conn, tx));

        Ok(Self { handle, recv: rx })
    }

    fn connect_to_server(config: &Config) -> Result<Self, conn::Error> {
        let conn = std::net::TcpStream::connect(&config.address)?;

        let (tx, rx) = channel::unbounded();
        let handle = std::thread::spawn(move || {
            let mut lines = BufReader::new(conn).lines();
            while let Some(Ok(line)) = lines.next() {
                let msg: streamchat::Message = serde_json::from_str(&line).expect("valid json");
                if tx.send(msg.into()).is_err() {
                    break;
                }
            }
            Ok(())
        });

        Ok(Self { handle, recv: rx })
    }

    fn wait_for_end(self) -> Result<(), conn::Error> {
        self.handle.join().unwrap() // thread unwind
    }
}
