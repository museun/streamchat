use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::sync::mpsc;

use super::{Error, Message};

pub fn connect(addr: &str) -> Result<mpsc::Receiver<Message>, Error> {
    let conn = TcpStream::connect(&addr).map_err(|e| {
        eprintln!("cannot connect to: {}", &addr);
        Error::Connect(e)
    })?;
    let (tx, rx) = mpsc::sync_channel(1);

    std::thread::spawn(move || {
        let mut lines = BufReader::new(conn).lines();
        while let Some(Ok(line)) = lines.next() {
            let msg = serde_json::from_str(&line).expect("valid json");
            tx.send(msg).unwrap()
        }
    });

    Ok(rx)
}
