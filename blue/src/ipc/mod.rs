pub mod receiver;
pub mod sender;
pub mod message {
    include!(concat!(env!("OUT_DIR"), "/blue.ipc.message.rs"));
}
