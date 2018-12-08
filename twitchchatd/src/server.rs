use super::*;

pub struct Server<C: Conn> {
    transports: Vec<Box<dyn Transport>>,
    conn: C,
}

impl<C: Conn> Server<C> {
    pub fn new(conn: C, transports: Vec<Box<dyn Transport>>) -> Self {
        Self { transports, conn }
    }

    pub fn run(&mut self) {
        while let Some(maybe) = self.conn.try_read() {
            if self.handle(&maybe).is_none() {
                info!("ending run loop");
                break;
            }
        }
    }

    fn handle(&mut self, maybe: &Maybe) -> Option<()> {
        match maybe {
            Maybe::Just(data) => {
                let msg = IrcMessage::parse(&data)?;
                match msg.command {
                    Command::Ping { data } => {
                        self.conn.write(&format!("PING {}", data)).ok()?;
                    }
                    Command::Privmsg { .. } => {
                        if let Some(msg) = msg.try_into_msg() {
                            self.dispatch(&msg);
                        }
                    }
                    Command::Unknown { cmd, args, data } => {
                        // TODO catch the NOTICE incorrect password
                        trace!("unknown msg: {} [{:?}] :{}", cmd, args, data)
                    }
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
