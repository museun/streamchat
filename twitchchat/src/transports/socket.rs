use super::*;

use std::io;
use std::io::prelude::*;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;

use crossbeam_channel as channel;

#[derive(Debug)]
pub struct Socket {
    tx: channel::Sender<Message>,
    rx: channel::Receiver<Message>,
    max: usize,
}

impl Socket {
    pub fn start(addr: &str, max: usize) -> Self {
        let (tx, rx) = channel::bounded(max);
        Self::run_loop(rx.clone(), addr, max);
        Self { tx, rx, max }
    }

    fn run_loop(rx: channel::Receiver<Message>, addr: &str, size: usize) {
        #[derive(Debug)]
        struct Client {
            id: u8,
            last: u64,
            stream: TcpStream,
        }

        let listener = TcpListener::bind(addr).expect("listen");
        listener
            .set_nonblocking(true)
            .expect("nonblocking mode must be set");

        thread::spawn(move || {
            let mut queue = Queue::new(size);
            let (mut clients, mut alive) = (vec![], vec![]);

            loop {
                'accept: loop {
                    match listener.accept() {
                        Ok((stream, _addr)) => {
                            let client = Client {
                                id: clients.len() as u8,
                                last: 0,
                                stream,
                            };
                            clients.push(client);
                            break 'accept;
                        }
                        Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => {}
                        Err(err) => eprintln!("error accepting client: {}", err),
                    }

                    if let Ok(msg) = rx.try_recv() {
                        let ts = msg.timestamp.clone();
                        let msg = serde_json::to_string(&msg).expect("valid json") + "\n";
                        queue.push((ts, msg));
                        break 'accept;
                    }

                    // TODO spin waiting is bad, use a semaphore or something like that
                    // clients likely won't be coming in often, so 150ms is fine
                    thread::park_timeout(std::time::Duration::from_millis(150));
                }

                'drain: for client in clients.drain(..) {
                    let mut client = client;
                    let last = client.last;
                    for msg in queue
                        .iter()
                        .filter_map(|(ts, m)| Some((u64::from_str_radix(&ts, 10).ok()?, m)))
                        .filter(|(ts, _)| *ts > last)
                        .map(|(_, m)| m)
                    {
                        if let Err(_err) = client.stream.write_all(msg.as_bytes()) {
                            let _ = client.stream.shutdown(Shutdown::Both);
                            continue 'drain;
                        }
                    }

                    if let Err(_err) = client.stream.flush() {
                        let _ = client.stream.shutdown(Shutdown::Both);
                        continue 'drain;
                    }

                    client.last = make_timestamp();
                    alive.push(client)
                }

                std::mem::swap(&mut clients, &mut alive);
                clients.shrink_to_fit();
            }
        });
    }
}

impl Transport for Socket {
    fn send(&mut self, data: &Message) {
        if self.rx.is_full() {
            self.rx.recv().unwrap(); // TODO handle this
        }
        // expensive..
        self.tx.send(data.clone()).unwrap(); // TODO handle this
    }
}
