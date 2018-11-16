use super::*;

#[derive(Debug, PartialEq)]
pub enum Maybe {
    Just(String),
    Nothing,
}

pub trait Conn {
    fn try_read(&mut self) -> Option<Maybe>;
    fn write(&mut self, data: &str) -> Result<usize, Error>;
}
