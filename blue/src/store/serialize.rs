use std::fs;
use std::io;
use std::path::Path;

use prost::Message;

use super::super::ipc::message;

pub fn serialize_store(store: &message::Store) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.reserve(store.encoded_len());
    store.encode(&mut buf).unwrap();
    buf
}

pub fn serialize_message<M: Message>(message: M) -> io::Result<Vec<u8>> {
    // let mut buf = Vec::new();
    // buf.reserve(message.encoded_len());
    // message.encode(&mut buf)?;
    let buf = message.encode_to_vec();
    Ok(buf)
}

pub fn serialize_message_with_len<M: Message>(message: M) -> io::Result<Vec<u8>> {
    let buf = message.encode_length_delimited_to_vec();
    Ok(buf)
}

pub fn persist_store(store: &mut message::Store, path: &Path) -> io::Result<()> {
    let bytes = serialize_store(store);
    fs::write(&path, &bytes)?;
    Ok(())
}
