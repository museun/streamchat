use crate::queue::Queue;
use crate::{Message, Transport};

use std::io::{self, prelude::*};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;

use crossbeam_channel as channel;
use log::{debug, info, trace, warn};

#[derive(Debug)]
pub struct Socket {
    tx: channel::Sender<Message>,
    rx: channel::Receiver<Message>,
    max: usize,
}

impl Socket {
    pub fn start(addr: &str, max: usize) -> Self {
        let (tx, rx) = channel::bounded(max);
        trace!("starting run loop, max of {} on {}", max, addr);
        Self::run_loop(rx.clone(), addr, max);
        Self { tx, rx, max }
    }

    fn run_loop(rx: channel::Receiver<Message>, addr: &str, size: usize) {
        struct Client {
            id: u8,
            last: u64,
            stream: TcpStream,
        }

        let listener = TcpListener::bind(addr).expect("listen");
        listener
            .set_nonblocking(true)
            .expect("nonblocking mode must be set");

        debug!(
            "socket transport listening on: {}",
            listener.local_addr().expect("get listeners local address")
        );

        thread::spawn(move || {
            let mut queue = Queue::new(size);
            let (mut clients, mut alive) = (vec![], vec![]);

            debug!("starting run loop");
            loop {
                'accept: loop {
                    match listener.accept() {
                        Ok((stream, addr)) => {
                            let client = Client {
                                id: clients.len() as u8,
                                last: 0,
                                stream,
                            };
                            info!("accepted client from: {}", addr);
                            clients.push(client);
                            break 'accept;
                        }
                        Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => {}
                        Err(err) => warn!("error accepting client: {}", err),
                    }

                    if let Ok(msg) = rx.try_recv() {
                        let ts = msg.timestamp.clone();
                        let msg = serde_json::to_string(&msg).expect("valid json") + "\n";
                        queue.push((ts, msg));
                        break 'accept;
                    }

                    // TODO: this should probably still be a park and not a yield
                    // TODO: look into crossbeams backup and/or parker
                    thread::park_timeout(std::time::Duration::from_millis(100))
                }

                'drain: for client in clients.drain(..) {
                    let mut client = client;

                    // if we cannot write to client, mark it as dead
                    // (e.g. skip adding it to the alive list)
                    macro_rules! try_client {
                        ($f:expr) => {
                            if let Err(_err) = $f() {
                                debug!("client appears to be disconnected: {}", client.id);
                                let _ = client.stream.shutdown(Shutdown::Both);
                                continue 'drain;
                            }
                        };
                    }

                    let last = client.last;
                    for msg in queue
                        .iter()
                        .filter_map(|(ts, m)| Some((u64::from_str_radix(&ts, 10).ok()?, m)))
                        .filter(|(ts, _)| *ts > last)
                        .map(|(_, m)| m)
                    {
                        try_client!(|| client.stream.write_all(msg.as_bytes()));
                    }

                    try_client!(|| client.stream.flush());

                    client.last = crate::make_timestamp();
                    alive.push(client)
                }

                trace!("new client list count: {}", alive.len());
                std::mem::swap(&mut clients, &mut alive);
                clients.shrink_to_fit();
            }
        });
    }
}

impl Transport for Socket {
    fn send(&mut self, data: Message) -> Result<(), Box<std::error::Error>> {
        use std::io::{Error, ErrorKind};

        if self.rx.is_full() {
            trace!("buffer full, dropping one");
            self.rx
                .recv()
                .map_err(|e| Box::new(Error::new(ErrorKind::NotConnected, e)))
                .map_err(|e| e as Box<dyn std::error::Error>)?;
        }

        self.tx
            .send(data)
            .map_err(|e| Box::new(Error::new(ErrorKind::Interrupted, e)))
            .map_err(|e| e as Box<dyn std::error::Error>)
    }
}
