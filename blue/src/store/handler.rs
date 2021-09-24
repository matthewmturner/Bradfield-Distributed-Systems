use std::io::{self, BufRead, BufReader, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use bytes::{Buf, BufMut, BytesMut};
use prost::Message;
use serde_json::json;

use super::super::ipc::message;
use super::super::ipc::message::request::Command;
use super::serialize::persist_store;

pub fn handle_stream(
    mut stream: TcpStream,
    store: Arc<Mutex<message::Store>>,
    store_path: Arc<&Path>,
) -> io::Result<()> {
    println!("Connection established with: {}", stream.peer_addr()?);
    let welcome = "Welcome to Blue!\n";
    stream.write(welcome.as_bytes())?;

    loop {
        // let input = parse_stream_input(&mut stream)?;
        let input = read_pb_message(&mut stream)?;
        println!("{:?}", input);
        let mut store = store.lock().unwrap();

        match input.command {
            Some(Command::Get(c)) => pb_get_handler(&mut stream, c, &mut store)?,
            Some(Command::Set(c)) => {
                pb_set_handler(&mut stream, c, &mut store)?;
                persist_store(&mut store, &store_path)?;
            }
            None => println!("Figure this out"),
        }

        // match input.command {
        //     Some(Command::Get) => get_handler(&mut stream, input, &mut store)?,
        //     Some(Command::Set) => {
        //         set_handler(&mut stream, input, &mut store)?;
        //         persist_store(&mut store, &store_path)?;
        //     }
        //     None => println!("Figure this out"),
        // }
    }
}

// fn parse_stream_input(stream: &mut TcpStream) -> io::Result<UserInput> {
//     let mut reader = BufReader::new(stream);
//     let mut line = String::new();
//     let bytes_read = reader.read_line(&mut line)?;
//     println!("Read {} bytes", bytes_read);

//     let user_input = UserInput::new(line);
//     if !user_input.is_valid {
//         println!("Invalid input")
//     }
//     Ok(user_input)
// }

// fn read_pb_message(stream: &mut TcpStream) -> io::Result<message::Request> {
//     let mut buf = BytesMut::with_capacity(1024);
//     // let mut buf: Vec<u8> = Vec::new();
//     let bytes_read = stream.read(&mut buf)?;
//     let user_input = message::Request::decode(&mut buf)?;
//     println!("Read {} bytes", bytes_read);
//     Ok(user_input)
// }

fn read_pb_message(stream: &mut TcpStream) -> io::Result<message::Request> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let len = i32::from_le_bytes(len_buf);
    println!("Length: {}", len);
    // let message = Vec::with_capacity(len as usize);
    // let mut message = BytesMut::with_capacity(len as usize);
    let mut buf = vec![0u8; len as usize];
    // stream.read_exact(&mut message)?;
    stream.read_exact(&mut buf)?;
    println!("Buf: {:?}", buf);
    let user_input = message::Request::decode(&mut buf.as_slice())?;
    // println!("Read {} bytes", bytes_read);
    Ok(user_input)
}

// fn get_handler(
//     stream: &mut TcpStream,
//     input: UserInput,
//     store: &mut message::Store,
// ) -> io::Result<()> {
//     // TODO: Get from store or file
//     if let Some(key) = input.value {
//         println!("Getting key={}", key);
//         let value = store.records.get(&key);
//         if let Some(v) = value {
//             let msg = format!("{}\n", v);
//             stream.write(msg.as_bytes())?;
//         };
//     } else {
//         println!("Getting store");
//         let msg = format!("{}\n", json!(store.records).to_string());
//         stream.write(msg.as_bytes())?;
//     }
//     Ok(())
// }

fn pb_get_handler(
    stream: &mut TcpStream,
    get: message::Get,
    store: &mut message::Store,
) -> io::Result<()> {
    // TODO: Get from store or file
    println!("Getting key={}", get.key);
    let value = store.records.get(&get.key);
    if let Some(v) = value {
        let msg = format!("{}\n", v);
        stream.write(msg.as_bytes())?;
    } else {
        println!("Getting store");
        let msg = format!("{}\n", json!(store.records).to_string());
        stream.write(msg.as_bytes())?;
    }
    Ok(())
}

// fn set_handler(
//     stream: &mut TcpStream,
//     input: UserInput,
//     store: &mut message::Store,
// ) -> io::Result<()> {
//     let set_value = input.value.expect("No set value provided");

//     let tokens: Vec<&str> = set_value.split("=").collect();

//     if tokens.len() == 2 {
//         let key = tokens[0];
//         let value = tokens[1].trim();
//         store.records.insert(key.to_string(), value.to_string());
//         println!("Storing {}={}", key, value);
//         let msg = format!("Succesfully wrote key={}\n", key);
//         stream.write(msg.as_bytes())?;
//         return Ok(());
//     }
//     Err(io::Error::new(
//         ErrorKind::InvalidData,
//         "Set commands must be of format key=val",
//     ))
// }

fn pb_set_handler(
    stream: &mut TcpStream,
    set: message::Set,
    store: &mut message::Store,
) -> io::Result<()> {
    // let set_value = input.value.expect("No set value provided");

    // let tokens: Vec<&str> = set_value.split("=").collect();

    // if tokens.len() == 2 {
    // let key = tokens[0];
    // let value = tokens[1].trim();
    println!("Storing {}={}", set.key, set.value);
    store.records.insert(set.key, set.value);
    let msg = "Succesfully wrote key to in memory story\n";
    stream.write(msg.as_bytes())?;
    Ok(())
}

// #[derive(Debug)]
// pub enum Command {
//     Get,
//     Set,
// }

// impl FromStr for Command {
//     type Err = ();
//     fn from_str(input: &str) -> Result<Command, Self::Err> {
//         match input {
//             "get" | "Get" | "GET" => Ok(Command::Get),
//             "set" | "Set" | "SET " => Ok(Command::Set),
//             _ => Err(()),
//         }
//     }
// }
// pub struct UserInput {
//     pub text: String,
//     pub is_valid: bool,
//     pub command: Option<Command>,
//     pub value: Option<String>,
// }

// impl UserInput {
//     pub fn new(input: String) -> UserInput {
//         let (validity, command, value) = UserInput::validate(&input);
//         match validity {
//             true => UserInput {
//                 text: input,
//                 is_valid: true,
//                 command,
//                 value,
//             },
//             false => UserInput {
//                 text: input,
//                 is_valid: false,
//                 command: None,
//                 value: None,
//             },
//         }
//     }
//     fn validate(input: &str) -> (bool, Option<Command>, Option<String>) {
//         let commands = vec!["get", "set"];
//         let tokens: Vec<&str> = input.split(' ').collect();

//         let validity = match tokens.len() {
//             0 => (false, None, None),
//             1 => {
//                 let command = tokens[0].trim();
//                 if command == "get" {
//                     (true, Some(Command::Get), None)
//                 } else {
//                     (false, None, None)
//                 }
//             }
//             2 => {
//                 let command = tokens[0].trim();
//                 if commands.iter().any(|&c| c == command) {
//                     (
//                         true,
//                         Some(Command::from_str(&command).unwrap()),
//                         Some(tokens[1].trim().to_string()),
//                     )
//                 } else {
//                     (false, None, None)
//                 }
//             }
//             _ => (false, None, None),
//         };
//         validity
//     }
// }
