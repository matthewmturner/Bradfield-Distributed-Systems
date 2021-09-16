use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::{self, BufRead, ErrorKind, Write};
use std::path::Path;
use std::str::FromStr;

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

fn receive_input() {}

fn parse_input(input_num: &mut i32, input: &mut String) -> io::Result<UserInput> {
    print!("[{}] ", input_num);
    io::stdout().flush()?;
    io::stdin().lock().read_line(input)?;
    let user_input = UserInput::new(input.to_string());
    if !user_input.is_valid {
        println!("Invalid input\n")
    }
    Ok(user_input)
}

fn get_input(input: UserInput, store: &mut HashMap<String, String>) -> io::Result<()> {
    if let Some(key) = input.value {
        let value = store.get(&key);
        if let Some(v) = value {
            println!("{}", v);
        };
    } else {
        println!("{:?}", store)
    }
    Ok(())
}

fn store_input(input: UserInput, store: &mut HashMap<String, String>) -> io::Result<()> {
    let set_value = input.value.expect("No set value provided");
    let tokens: Vec<&str> = set_value.split("=").collect();

    if tokens.len() == 2 {
        let key = tokens[0];
        let value = tokens[1].trim();
        store.insert(key.to_string(), value.to_string());
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

fn main() -> Result<(), Box<dyn Error>> {
    let mut input_num: i32 = 1;
    let store_path = Path::new("data.json");

    let mut store: HashMap<String, String> = match store_path.exists() {
        true => {
            let existing_store = fs::read_to_string(&store_path)?;
            serde_json::from_str(&existing_store)?
        }
        false => HashMap::new(),
    };

    loop {
        let mut input = String::new();
        let input = parse_input(&mut input_num, &mut input)?;
        match input.command {
            Some(Command::Get) => get_input(input, &mut store)?,
            Some(Command::Set) => {
                store_input(input, &mut store)?;
                persist_store(&mut store, store_path)?;
            }
            None => println!("Figure this out"),
        }
        input_num += 1;
    }
}
