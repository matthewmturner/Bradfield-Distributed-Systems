use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

use prost::Message;
use serde_json::json;

use super::super::ipc::message;

pub fn serialize_store(store: &message::Store) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.reserve(store.encoded_len());
    store.encode(&mut buf).unwrap();
    buf
}

pub fn persist_store(store: &mut HashMap<String, String>, path: &Path) -> io::Result<()> {
    let json = json!(store);
    fs::write(path, json.to_string())?;
    Ok(())
}
