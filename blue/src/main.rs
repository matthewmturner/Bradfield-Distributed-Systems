use std::collections::HashMap;
use std::error::Error;
use std::io::{self, BufRead, Write};
use std::str::FromStr;

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
            1 | 2 => {
                let command = tokens[0].replace("\n", "");
                if commands.iter().any(|&c| c == command.as_str()) {
                    (
                        true,
                        Some(Command::from_str(&command).unwrap()),
                        Some(tokens[1].to_string()),
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
    match user_input.is_valid {
        true => println!("{}", input),
        false => println!("Invalid input\n"),
    }
    Ok(user_input)
}

fn get_input(input: UserInput, store: &mut HashMap<String, String>) {}

fn store_input(input: UserInput, store: &mut HashMap<String, String>) {}

fn main() -> Result<(), Box<dyn Error>> {
    let mut input_num: i32 = 1;
    let mut store: HashMap<String, String> = HashMap::new();

    loop {
        let mut input = String::new();
        let input = parse_input(&mut input_num, &mut input)?;
        match input.command {
            Some(Command::Get) => get_input(input, &mut store),
            Some(Command::Set) => store_input(input, &mut store),
            None => println!("Figure this out"),
        }
        input_num += 1;
    }
}
