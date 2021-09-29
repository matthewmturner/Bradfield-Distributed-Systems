use std::fs::{File, OpenOptions};
use std::io::{self, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use prost::Message;

use super::serialize::serialize_message_with_len;

static WAL_VERSION: u8 = 1;
static PROTO_BUF_VERSION: u8 = 3;

pub struct WriteAheadLog<'a> {
    path: &'a Path,
    file: File,
    pub next_sequence: u64,
}

impl<'a> WriteAheadLog<'a> {
    pub fn new(path: &'a PathBuf) -> io::Result<WriteAheadLog> {
        let magic = b"BLUE";
        let header = [WAL_VERSION, PROTO_BUF_VERSION];
        let sequence = 1u64.to_le_bytes();
        let mut file = File::create(path)?;
        file.write_all(magic)?;
        file.write_all(&header)?;
        file.write_all(&sequence)?;
        Ok(WriteAheadLog {
            path,
            file,
            next_sequence: 1u64,
        })
    }
    pub fn open(path: &'a PathBuf) -> io::Result<WriteAheadLog> {
        let mut file = File::open(path)?;
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)?;
        let metadata = file.metadata()?;
        let size = metadata.len();
        file.seek(SeekFrom::Start(size - 8))?;
        let mut sequence_bytes = [0u8; 8];
        file.read_exact(&mut sequence_bytes)?;
        match &magic == b"BLUE" {
            true => Ok(WriteAheadLog {
                path,
                file,
                next_sequence: u64::from_le_bytes(sequence_bytes),
            }),
            false => Err(io::Error::new(
                ErrorKind::InvalidData,
                "Invalid magic number in WAL",
            )),
        }
    }
    pub fn append_message<M: Message>(&mut self, message: M) -> io::Result<()> {
        let bytes = serialize_message_with_len(message)?;
        let mut file = OpenOptions::new().append(true).open("wal0000.log")?;
        file.write_all(&bytes)?;
        self.next_sequence += 1;
        file.write_all(&self.next_sequence.to_le_bytes())?;
        Ok(())
    }
}
