use std::collections::HashMap;
use std::error::Error;
use std::io::{self, BufRead, Write};

struct UserInput {
    text: String,
    is_valid: bool,
    command: Option<String>,
}

impl UserInput {
    fn new(input: String) -> UserInput {
        let (validity, command) = UserInput::validate(&input);
        match validity {
            true => UserInput {
                text: input,
                is_valid: true,
                command,
            },
            false => UserInput {
                text: input,
                is_valid: false,
                command: None,
            },
        }
    }
    fn validate(input: &str) -> (bool, Option<String>) {
        let commands = vec!["get", "set"];
        let tokens: Vec<&str> = input.split(' ').collect();

        let validity = match tokens.len() {
            0 => (false, None),
            1 | 2 => {
                let command = tokens[0].replace("\n", "");
                if commands.iter().any(|&c| c == command.as_str()) {
                    (true, Some(tokens[0].to_string()))
                } else {
                    (false, None)
                }
            }
            _ => (false, None),
        };
        validity
    }
}

fn validate_input(input_num: &mut i32, input: &mut String) -> io::Result<UserInput> {
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

fn main() -> Result<(), Box<dyn Error>> {
    let mut input_num: i32 = 1;

    loop {
        let mut input = String::new();
        validate_input(&mut input_num, &mut input);
        input_num += 1;
    }
}
