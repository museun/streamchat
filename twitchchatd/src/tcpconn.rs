use super::*;

use std::io::{BufRead, BufReader, BufWriter, Lines, Write};
use std::net::TcpStream;
use std::str;

pub struct TcpConn {
    reader: Lines<BufReader<TcpStream>>,
    writer: BufWriter<TcpStream>,
}

impl TcpConn {
    pub fn connect(addr: &str, token: &str, channel: &str, nick: &str) -> Result<Self, Error> {
        info!("connecting to twitch for #{} / {}", channel, nick);
        macro_rules! maybe {
            ($e:expr) => {
                $e.map_err(|_e| Error::CannotConnect)?
            };
        };

        let conn = maybe!(TcpStream::connect(&addr));
        // TODO don't do this
        maybe!(conn.set_read_timeout(Some(std::time::Duration::from_millis(100))));

        let reader = BufReader::new(maybe!(conn.try_clone())).lines();
        let writer = BufWriter::new(maybe!(conn.try_clone()));

        let pass = format!("PASS {}", &token);
        let nick = format!("NICK {}", &nick);
        let join = format!("JOIN #{}", &channel);

        let mut this = Self { reader, writer };

        maybe!(this.write("CAP REQ :twitch.tv/tags"));
        maybe!(this.write("CAP REQ :twitch.tv/membership"));
        maybe!(this.write("CAP REQ :twitch.tv/commands"));
        maybe!(this.write(&pass));
        maybe!(this.write(&nick));
        maybe!(this.write(&join));

        info!("connected to twitch");
        Ok(this)
    }
}

impl Conn for TcpConn {
    fn try_read(&mut self) -> Option<Maybe> {
        use std::io::ErrorKind::*;
        match self.reader.next() {
            Some(Ok(line)) => Some(Maybe::Just(line)),
            Some(Err(ref e)) if e.kind() == WouldBlock || e.kind() == TimedOut => {
                Some(Maybe::Nothing)
            }
            Some(Err(..)) | None => None,
        }
    }

    fn write(&mut self, input: &str) -> Result<usize, Error> {
        let mut n = 0usize;
        for data in split(input).iter().map(|s| s.as_bytes()) {
            self.writer
                .write_all(&data)
                .map_err(|_e| Error::CannotWrite)?;
            n += data.len();
        }
        // assert some post condition that n == some number
        self.writer.flush().map_err(|_e| Error::CannotWrite)?;
        Ok(n)
    }
}

#[inline]
fn split(raw: &str) -> Vec<String> {
    if raw.len() > 510 && raw.contains(':') {
        let mut split = raw.splitn(2, ':').map(str::trim);
        let (head, tail) = (split.next().unwrap(), split.next().unwrap());
        return tail
            .as_bytes()
            .chunks(510 - head.len())
            .map(str::from_utf8)
            .filter_map(|s| s.ok())
            .map(|s| format!("{} :{}\r\n", head, s))
            .collect();
    }
    vec![format!("{}\r\n", raw)]
}
