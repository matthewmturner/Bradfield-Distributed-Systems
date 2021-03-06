use std::fs::{File, OpenOptions};
use std::io::{self, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use log::debug;
use prost::Message;

use super::super::ipc::message;

static WAL_VERSION: u8 = 1;
static PROTO_BUF_VERSION: u8 = 3;

type Sequence = u64;
pub type WalItem = (Sequence, message::Set);

#[derive(Debug, Clone)]
pub struct WriteAheadLog {
    path: PathBuf,
    pub next_sequence: u64, // Next sequence number to be appended
}

impl<'a> WriteAheadLog {
    pub fn new(path: &Path) -> io::Result<WriteAheadLog> {
        let magic = b"BLUE";
        let header = [WAL_VERSION, PROTO_BUF_VERSION];
        let sequence = 1u64.to_le_bytes();
        let mut file = File::create(path)?;
        // Do the below in one write call to minimize sys calls
        file.write_all(magic)?;
        file.write_all(&header)?;
        file.write_all(&sequence)?;
        Ok(WriteAheadLog {
            path: path.to_path_buf(),
            next_sequence: 1u64,
        })
    }
    pub fn open(path: &Path) -> io::Result<WriteAheadLog> {
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
                path: path.to_path_buf(),
                next_sequence: u64::from_le_bytes(sequence_bytes),
            }),
            false => Err(io::Error::new(
                ErrorKind::InvalidData,
                "Invalid magic number in WAL",
            )),
        }
    }
    pub fn append_message<M: Message>(&mut self, message: &M) -> io::Result<()> {
        debug!("Appending msg to wal: {:?}", message);
        let bytes = message.encode_length_delimited_to_vec();
        let mut file = OpenOptions::new().append(true).open(&self.path)?;
        file.write_all(&bytes)?;
        self.next_sequence += 1;
        file.write_all(&self.next_sequence.to_le_bytes())?;
        Ok(())
    }

    pub fn messages(self) -> io::Result<Vec<WalItem>> {
        let mut file = File::open(&self.path)?;
        let file_len = file.metadata().unwrap().len();
        file.seek(SeekFrom::Start(6))?;

        let mut msgs: Vec<WalItem> = Vec::new();
        if self.next_sequence == 1 {
            return Ok(msgs);
        }

        loop {
            let mut sequence_buf = [0u8; 8];
            file.read_exact(&mut sequence_buf)?;
            let sequence = u64::from_le_bytes(sequence_buf);
            let mut len_buf = [0u8; 1];
            file.read_exact(&mut len_buf)?;
            let len = u8::from_le_bytes(len_buf);
            let mut msg_buf = vec![0u8; len as usize];
            Read::by_ref(&mut file).read_exact(&mut msg_buf)?;
            let msg = message::Set::decode(&mut msg_buf.as_slice())?;
            msgs.push((sequence, msg));
            let pos = file.stream_position()?;
            if (file_len - pos) == 8 {
                break;
            }
        }

        Ok(msgs)
    }
}
