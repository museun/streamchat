use twitchchat::prelude::*;

use std::env;
use std::io::prelude::*;
use std::io::BufReader;
use std::net::TcpStream;

use termcolor::{BufferWriter, ColorChoice};

mod buffer;
mod error;
mod options;

use self::buffer::Buffer;
use self::error::Error;
use self::options::Options;

fn main() {
    let opts = Options::parse(&env::args().collect::<Vec<_>>());
    if let Err(err) = connect(&opts) {
        die(format!("client error: {}", err))
    }
}

fn connect(opts: &Options) -> Result<(), Error> {
    let conn = TcpStream::connect(&opts.addr).map_err(|_e| Error::CannotConnect)?;
    let reader = BufReader::new(conn).lines();

    let writer = BufferWriter::stdout(if opts.use_colors {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    });

    for line in reader {
        let line = line.map_err(|_e| Error::CannotRead)?;
        let msg = serde_json::from_str::<Message>(&line).expect("valid json");

        let mut buf = writer.buffer();
        Buffer::format(&opts, &msg, &mut buf);
        writer.print(&buf).expect("print");
    }
    Ok(())
}

pub(crate) fn die<S: AsRef<str>>(s: S) -> ! {
    eprintln!("{}", s.as_ref());
    std::process::exit(1)
}
