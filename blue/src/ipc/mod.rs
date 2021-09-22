pub mod reader;
pub mod writer;

pub mod message {
    include!(concat!(env!("OUT_DIR"), "/blue.ipc.message.rs"));
}
