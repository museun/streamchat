use crate::client::{Client, Event, Stream};
use crate::Transport;

use log::*;

pub struct Dispatcher<'a, S: Stream> {
    transports: &'a mut [&'a mut dyn Transport],
    client: Client<S>,
}

impl<'a, S: Stream> Dispatcher<'a, S> {
    pub fn new(client: Client<S>, transports: &'a mut [&'a mut dyn Transport]) -> Self {
        Self { transports, client }
    }

    pub fn run(self) {
        debug!("starting listener loop");
        for msg in self.client {
            if let Event::Privmsg(msg) = msg {
                for transport in self.transports.iter_mut() {
                    if let Err(err) = transport.send(&msg) {
                        // TODO maybe a collate errors to remove "bad" transports
                        warn!("cannot send to transport: {} -> {}", transport.name(), err)
                    }
                }
            }
        }
        debug!("end of listener loop")
    }
}
