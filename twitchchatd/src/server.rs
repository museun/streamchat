use super::*;

pub struct Server {
    transports: Vec<Box<dyn Transport>>,
    conn: Box<dyn Conn>, // this..
}

impl Server {
    pub fn new(conn: Box<dyn Conn>, transports: Vec<Box<dyn Transport>>) -> Self {
        Self { transports, conn }
    }

    pub fn run(&mut self) {
        while let Some(maybe) = self.conn.try_read() {
            eprintln!(">> {:?}", maybe);
            if self.handle(&maybe).is_none() {
                break;
            }
        }
    }

    fn handle(&mut self, maybe: &Maybe) -> Option<()> {
        match maybe {
            Maybe::Just(data) => {
                let msg = IrcMessage::parse(&data)?;
                eprintln!("msg: {:?}", msg);
                match msg.command {
                    Command::Ping { data } => {
                        self.conn.write(&format!("PING {}", data)).ok()?;
                    }
                    Command::Privmsg { .. } => {
                        if let Some(msg) = msg.try_into_msg() {
                            self.dispatch(&msg);
                        }
                    }
                    Command::Unknown { .. } => {}
                }
            }
            Maybe::Nothing => {}
        };
        Some(())
    }

    fn dispatch(&mut self, msg: &Message) {
        for transport in &mut self.transports {
            transport.send(&msg)
        }
    }
}
