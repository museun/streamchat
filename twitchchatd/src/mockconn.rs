use super::*;
use std::io;

pub struct MockConn<T: Iterator<Item = Result<String, io::Error>>>(T);

impl<T> MockConn<T>
where
    T: Iterator<Item = Result<String, io::Error>>,
{
    pub fn new(source: T) -> Self {
        MockConn(source)
    }
}

impl<T> Conn for MockConn<T>
where
    T: Iterator<Item = Result<String, io::Error>>,
{
    fn try_read(&mut self) -> Option<Maybe> {
        match self.0.next() {
            Some(Ok(string)) => Some(Maybe::Just(string)),
            _ => {
                // to slow spinning
                std::thread::park_timeout(std::time::Duration::from_millis(100));
                Some(Maybe::Nothing)
            }
        }
    }

    fn write(&mut self, _data: &str) -> Result<usize, Error> {
        warn!("writing to a mock conn does nothing");
        Ok(0) // do nothing
    }
}
