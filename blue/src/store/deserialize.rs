use std::fs;
use std::io::Cursor;
use std::path::Path;

use prost::Message;

use super::super::ipc::message;

pub fn deserialize_store(path: &Path) -> Result<message::Store, prost::DecodeError> {
    let store = match path.exists() {
        true => {
            let existing_store = fs::read(&path).unwrap();
            message::Store::decode(&mut Cursor::new(existing_store.as_slice()))
        }
        false => Ok(message::Store::default()),
    };
    store
}
