use super::*;

use std::fs::File as RFile;
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::Path;

// TODO this needs some actual error handling
pub struct File {
    formatted: bool,
    file: BufWriter<RFile>,
}

impl File {
    pub fn create(name: impl AsRef<Path>, formatted: bool) -> Self {
        trace!(
            "created file (formatted={}) transport for: {}",
            formatted,
            name.as_ref().to_string_lossy()
        );
        Self {
            formatted,
            file: BufWriter::new(RFile::create(name).unwrap()),
        }
    }
}

impl Transport for File {
    fn send(&mut self, data: &Message) {
        if self.formatted {
            writeln!(self.file, "{}: {}", data.name, data.data).unwrap();
        } else {
            writeln!(self.file, "{}", serde_json::to_string(&data).unwrap()).unwrap();
        }
        self.file.flush().unwrap();
    }
}
