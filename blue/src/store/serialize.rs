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

pub fn persist_store(store: &mut message::Store, path: &Path) -> io::Result<()> {
    let bytes = serialize_store(&store);
    fs::write(&path, &bytes)?;
    Ok(())
}
