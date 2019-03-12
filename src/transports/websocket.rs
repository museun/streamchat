use crate::*;
use log::*;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Config {
    address: String,
}

impl Configurable for Config {
    fn name() -> &'static str {
        "websocket.toml"
    }
}

#[derive(Debug)]
pub enum Error {
    Create(ws::Error),
    Listen(ws::Error),
    Send(ws::Error),
    Serde(serde_json::Error),
    RestartRequired,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use self::Error::*;
        match self {
            Create(err) => write!(f, "cannot create websocket: {}", err),
            Listen(err) => write!(f, "cannot listen/bind for websocket: {}", err),
            Send(err) => write!(f, "cannot send a message to the websocket: {}", err),
            Serde(err) => write!(f, "cannot serialize the json: {}", err),
            RestartRequired => write!(f, "a restart of the websocket server is required"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use self::Error::*;
        match self {
            Create(err) | Listen(err) | Send(err) => Some(err as &(dyn std::error::Error)),
            Serde(err) => Some(err as &(dyn std::error::Error)),
            _ => None,
        }
    }
}

static DEAD: AtomicBool = AtomicBool::new(false);

pub struct WebsocketServer {
    sender: ws::Sender,
}

impl WebsocketServer {
    pub fn start(config: Config) -> Result<Self, Error> {
        // TODO keep this from spawning zombies
        if DEAD.load(Ordering::Relaxed) {
            DEAD.store(false, Ordering::Relaxed);
        }

        let socket = ws::WebSocket::new(|_| |_| Ok(())).map_err(Error::Create)?;
        let sender = socket.broadcaster();

        let addr = config.address.clone();
        thread::spawn(move || {
            if let Err(err) = socket.listen(&addr).map_err(Error::Listen) {
                error!("cannot listen with websocket: {}", err);
                DEAD.store(true, Ordering::Relaxed);
            }
        });

        Ok(Self { sender })
    }
}

impl Transport for WebsocketServer {
    fn send(&mut self, msg: message::Message) -> Result<(), Box<std::error::Error>> {
        if DEAD.load(Ordering::Relaxed) {
            return Err(Error::RestartRequired)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>);
        }

        serde_json::to_string(&msg)
            .map_err(Error::Serde)
            .and_then(|json| {
                self.sender
                    .send(ws::Message::Text(json))
                    .map_err(Error::Send)
            })
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>) // neat
    }
}
