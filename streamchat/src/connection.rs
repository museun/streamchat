use crate::twitch;
use crossbeam_channel as channel;
use std::net::TcpStream;

// TODO pass in the reader so we can monomorphize
pub fn connect_to_twitch(
    nick: impl AsRef<str>,
    token: impl AsRef<str>,
    channel: impl AsRef<str>,
) -> Result<twitch::Client<twitch::SyncReadAdapter<TcpStream>>, Error> {
    let nick = nick.as_ref();
    let token = token.as_ref();
    let channel = channel.as_ref();

    if nick.is_empty() {
        return Err(Error::InvalidNick)?;
    }
    if token.is_empty() {
        return Err(Error::InvalidToken)?;
    }
    if channel.is_empty() {
        return Err(Error::InvalidChannel)?;
    }

    log::info!("connecting to: {}", twitch::TWITCH_IRC_ADDRESS);
    let (read, write) = {
        let read = TcpStream::connect(twitch::TWITCH_IRC_ADDRESS)?;
        let write = read.try_clone().expect("clone tcpstream");
        (read, write)
    };
    log::info!("opened connection");

    let (read, write) = twitch::sync_adapters(read, write);
    let mut client = twitch::Client::new(read, write);

    let conf = twitch::UserConfig::builder()
        .nick(nick)
        .token(token)
        .tags()
        .commands()
        .build()
        .expect("valid configuration");
    log::info!("registering with nick: {}", conf.nick);
    client.register(conf)?;

    let user = match client.wait_for_ready() {
        Ok(user) => user,
        Err(twitch::Error::InvalidRegistration) => return Err(Error::InvalidLogin),
        Err(err) => return Err(err.into()),
    };

    log::info!(
        "connected with {} ({}).",
        user.display_name.expect("get our display name"),
        user.user_id
    );

    client.writer().join(&channel)?;
    log::info!("joined: {}", channel.to_string());

    Ok(client)
}

pub fn read_until_end<R>(
    client: twitch::Client<R>,
    send: channel::Sender<crate::Message>,
) -> Result<(), Error>
where
    R: twitch::ReadAdapter + Send + Sync,
{
    let mut client = client;
    loop {
        let msg = match client.read_message()? {
            twitch::Message::PrivMsg(msg) => msg,
            _ => continue,
        };

        let msg = msg.into();
        if send.send(msg).is_err() {
            break;
        }
    }

    Err(Error::Disconnected)
}

#[derive(Debug)]
pub enum Error {
    Twitch(twitch::Error),
    Io(std::io::Error),
    Disconnected,
    InvalidLogin,
    InvalidNick,
    InvalidToken,
    InvalidChannel,
}

impl From<twitch::Error> for Error {
    fn from(err: twitch::Error) -> Self {
        Error::Twitch(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Twitch(err) => write!(f, "twitch error: {}", err),
            Error::Io(err) => write!(f, "io error: {}", err),
            Error::Disconnected => write!(f, "disconnected"),
            Error::InvalidLogin => write!(f, "invalid login"),
            Error::InvalidNick => write!(f, "invalid nick"),
            Error::InvalidToken => write!(f, "invalid token"),
            Error::InvalidChannel => write!(f, "invalid channel"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Twitch(err) => Some(err),
            Error::Io(err) => Some(err),
            _ => None,
        }
    }
}
