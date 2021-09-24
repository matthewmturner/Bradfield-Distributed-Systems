use std::io;
use std::net::TcpStream;
use std::path::Path;
use std::sync::{Arc, Mutex};

use serde_json::json;

use super::super::ipc::message;
use super::super::ipc::message::request::Command;
use super::super::ipc::receiver::read_message;
use super::super::ipc::sender::send_message;
use super::serialize::persist_store;

pub fn handle_stream(
    mut stream: TcpStream,
    store: Arc<Mutex<message::Store>>,
    store_path: Arc<&Path>,
) -> io::Result<()> {
    println!("Connection established with: {}", stream.peer_addr()?);
    let welcome = message::Welcome {
        message: "Welcome to Blue!\n".to_string(),
    };
    send_message(welcome, &mut stream)?;

    loop {
        let input = read_message::<message::Request>(&mut stream)?;
        println!("{:?}", input);
        let mut store = store.lock().unwrap();

        match input.command {
            Some(Command::Get(c)) => get_handler(&mut stream, c, &mut store)?,
            Some(Command::Set(c)) => {
                set_handler(&mut stream, c, &mut store)?;
                persist_store(&mut store, &store_path)?;
            }
            None => println!("Figure this out"),
        }
    }
}

fn get_handler(
    stream: &mut TcpStream,
    get: message::Get,
    store: &mut message::Store,
) -> io::Result<()> {
    // TODO: Get from store or file
    println!("Getting key={}", get.key);
    let value = store.records.get(&get.key);
    let msg = match value {
        Some(v) => message::Response {
            success: true,
            message: v.clone(),
        },
        None => message::Response {
            success: true,
            message: json!(store.records).to_string(),
        },
    };
    send_message(msg, stream)
}

fn set_handler(
    stream: &mut TcpStream,
    set: message::Set,
    store: &mut message::Store,
) -> io::Result<()> {
    println!("Storing {}={}", set.key, set.value);
    store.records.insert(set.key.clone(), set.value.clone());
    let msg = message::Response {
        success: true,
        message: "Succesfully wrote key to in memory story".to_string(),
    };
    send_message(msg, stream)?;

    Ok(())
}
