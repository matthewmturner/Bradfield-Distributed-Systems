use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::{self, BufRead, BufReader, ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::str::FromStr;
use std::thread;

use serde_json::json;

#[derive(Debug)]
enum Command {
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
struct UserInput {
    text: String,
    is_valid: bool,
    command: Option<Command>,
    value: Option<String>,
}

impl UserInput {
    fn new(input: String) -> UserInput {
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

fn parse_stream_input(stream: &mut TcpStream, input_num: &mut i32) -> io::Result<UserInput> {
    let msg = format!("[{}] ", input_num);
    stream.write(msg.as_bytes())?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let bytes_read = reader.read_line(&mut line)?;
    println!("Read {} bytes", bytes_read);

    let user_input = UserInput::new(line);
    if !user_input.is_valid {
        println!("Invalid input\n")
    }
    Ok(user_input)
}

fn get_input(
    stream: &mut TcpStream,
    input: UserInput,
    store: &mut HashMap<String, String>,
) -> io::Result<()> {
    if let Some(key) = input.value {
        println!("Getting key={}", key);
        let value = store.get(&key);
        if let Some(v) = value {
            let msg = format!("{}\n", v);
            stream.write(msg.as_bytes())?;
        };
    } else {
        println!("Getting store");
        let msg = format!("{}\n", json!(store).to_string());
        stream.write(msg.as_bytes())?;
    }
    Ok(())
}

fn store_stream_input(
    stream: &mut TcpStream,
    input: UserInput,
    store: &mut HashMap<String, String>,
) -> io::Result<()> {
    let set_value = input.value.expect("No set value provided");

    let tokens: Vec<&str> = set_value.split("=").collect();

    if tokens.len() == 2 {
        let key = tokens[0];
        let value = tokens[1].trim();
        store.insert(key.to_string(), value.to_string());
        let msg = format!("Succesfully wrote key={}\n", key);
        stream.write(msg.as_bytes())?;
        return Ok(());
    }
    Err(io::Error::new(
        ErrorKind::InvalidData,
        "Set commands must be of format key=val",
    ))
}

fn persist_store(store: &mut HashMap<String, String>, path: &Path) -> io::Result<()> {
    let json = json!(store);
    fs::write(path, json.to_string())?;
    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> io::Result<()> {
    let mut input_num: i32 = 1;
    let welcome = "Welcome to Blue!\n";
    stream.write(welcome.as_bytes())?;

    let store_path = Path::new("data.json");

    let mut store: HashMap<String, String> = match store_path.exists() {
        true => {
            let existing_store = fs::read_to_string(&store_path)?;
            serde_json::from_str(&existing_store)?
        }
        false => HashMap::new(),
    };

    loop {
        // let mut input = String::new();
        let input = parse_stream_input(&mut stream, &mut input_num)?;
        match input.command {
            Some(Command::Get) => get_input(&mut stream, input, &mut store)?,
            Some(Command::Set) => {
                store_stream_input(&mut stream, input, &mut store)?;
                persist_store(&mut store, store_path)?;
            }
            None => println!("Figure this out"),
        }
        input_num += 1;
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        thread::spawn(|| {
            handle_connection(stream);
        });
    }
    Ok(())
}
