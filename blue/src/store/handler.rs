use std::io::{self, BufRead, BufReader, ErrorKind, Write};
use std::net::TcpStream;
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use serde_json::json;

use super::super::ipc::message;
use super::serialize::persist_store;

// NOTE: Old handler pre multi threading
// pub fn handle_connection(mut stream: TcpStream) -> io::Result<()> {
//     println!("Connection established with: {}", stream.peer_addr()?);
//     let mut input_num: i32 = 1;
//     let welcome = "Welcome to Blue!\n";
//     stream.write(welcome.as_bytes())?;

//     let store_path = Path::new("data.json");

//     let mut store: HashMap<String, String> = match store_path.exists() {
//         true => {
//             let existing_store = fs::read_to_string(&store_path)?;
//             serde_json::from_str(&existing_store)?
//         }
//         false => HashMap::new(),
//     };

//     loop {
//         let input = parse_stream_input(&mut stream, &mut input_num)?;
//         match input.command {
//             Some(Command::Get) => get_hander(&mut stream, input, &mut store)?,
//             Some(Command::Set) => {
//                 set_handler(&mut stream, input, &mut store)?;
//                 persist_store(&mut store, store_path)?;
//             }
//             None => println!("Figure this out"),
//         }
//         input_num += 1;
//     }
// }

pub fn handle_stream(
    mut stream: TcpStream,
    store: Arc<Mutex<message::Store>>,
    store_path: Arc<&Path>,
) -> io::Result<()> {
    println!("Connection established with: {}", stream.peer_addr()?);
    let mut input_num: i32 = 1;
    let welcome = "Welcome to Blue!\n";
    stream.write(welcome.as_bytes())?;

    loop {
        let input = parse_stream_input(&mut stream, &mut input_num)?;
        let mut store = store.lock().unwrap();

        match input.command {
            Some(Command::Get) => get_hander(&mut stream, input, &mut store)?,
            Some(Command::Set) => {
                set_handler(&mut stream, input, &mut store)?;
                persist_store(&mut store, &store_path)?;
            }
            None => println!("Figure this out"),
        }
        input_num += 1;
    }
}

fn parse_stream_input(stream: &mut TcpStream, input_num: &mut i32) -> io::Result<UserInput> {
    let msg = format!("[{}] ", input_num);
    stream.write(msg.as_bytes())?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let bytes_read = reader.read_line(&mut line)?;
    println!("Read {} bytes", bytes_read);

    let user_input = UserInput::new(line);
    if !user_input.is_valid {
        println!("Invalid input")
    }
    Ok(user_input)
}

fn get_hander(
    stream: &mut TcpStream,
    input: UserInput,
    store: &mut message::Store,
) -> io::Result<()> {
    // TODO: Get from store or file
    if let Some(key) = input.value {
        println!("Getting key={}", key);
        let value = store.records.get(&key);
        if let Some(v) = value {
            let msg = format!("{}\n", v);
            stream.write(msg.as_bytes())?;
        };
    } else {
        println!("Getting store");
        let msg = format!("{}\n", json!(store.records).to_string());
        stream.write(msg.as_bytes())?;
    }
    Ok(())
}

fn set_handler(
    stream: &mut TcpStream,
    input: UserInput,
    store: &mut message::Store,
) -> io::Result<()> {
    let set_value = input.value.expect("No set value provided");

    let tokens: Vec<&str> = set_value.split("=").collect();

    if tokens.len() == 2 {
        let key = tokens[0];
        let value = tokens[1].trim();
        store.records.insert(key.to_string(), value.to_string());
        println!("Storing {}={}", key, value);
        let msg = format!("Succesfully wrote key={}\n", key);
        stream.write(msg.as_bytes())?;
        return Ok(());
    }
    Err(io::Error::new(
        ErrorKind::InvalidData,
        "Set commands must be of format key=val",
    ))
}

#[derive(Debug)]
pub enum Command {
    Get,
    Set,
}

impl FromStr for Command {
    type Err = ();
    fn from_str(input: &str) -> Result<Command, Self::Err> {
        match input {
            "get" | "Get" | "GET" => Ok(Command::Get),
            "set" | "Set" | "SET " => Ok(Command::Set),
            _ => Err(()),
        }
    }
}
pub struct UserInput {
    pub text: String,
    pub is_valid: bool,
    pub command: Option<Command>,
    pub value: Option<String>,
}

impl UserInput {
    pub fn new(input: String) -> UserInput {
        let (validity, command, value) = UserInput::validate(&input);
        match validity {
            true => UserInput {
                text: input,
                is_valid: true,
                command,
                value,
            },
            false => UserInput {
                text: input,
                is_valid: false,
                command: None,
                value: None,
            },
        }
    }
    fn validate(input: &str) -> (bool, Option<Command>, Option<String>) {
        let commands = vec!["get", "set"];
        let tokens: Vec<&str> = input.split(' ').collect();

        let validity = match tokens.len() {
            0 => (false, None, None),
            1 => {
                let command = tokens[0].trim();
                if command == "get" {
                    (true, Some(Command::Get), None)
                } else {
                    (false, None, None)
                }
            }
            2 => {
                let command = tokens[0].trim();
                if commands.iter().any(|&c| c == command) {
                    (
                        true,
                        Some(Command::from_str(&command).unwrap()),
                        Some(tokens[1].trim().to_string()),
                    )
                } else {
                    (false, None, None)
                }
            }
            _ => (false, None, None),
        };
        validity
    }
}
