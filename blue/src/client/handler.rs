use std::io::{self, BufRead, BufReader, ErrorKind, Stdin};
use std::net::SocketAddr;
use std::str::FromStr;

use super::super::ipc::message;
use super::super::ipc::message::request::Command;

pub fn read_client_request(stdin: &mut Stdin) -> io::Result<String> {
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    Ok(line)
}

pub fn parse_request(input: String) -> io::Result<message::Request> {
    let tokens: Vec<&str> = input.split(' ').collect();
    let command = extract_command(tokens)?;
    println!("{:?}", command);
    Ok(message::Request {
        command: Some(command),
    })
}

fn extract_command(tokens: Vec<&str>) -> io::Result<Command> {
    let command = match tokens[0].trim() {
        "get" | "Get" | "GET" => Ok(get_handler(&tokens)?),
        "set" | "Set" | "SET " => Ok(set_handler(&tokens)?),
        // "backup" | "Backup" | "BACKUP " => Ok(backup_handler(&tokens)?),
        _ => Err(io::Error::new(ErrorKind::InvalidData, "Invalid command")),
    };
    command
}

fn get_handler(tokens: &[&str]) -> io::Result<Command> {
    match tokens.len() {
        1 => Ok(Command::Get(message::Get::default())),
        2 => Ok(Command::Get(message::Get {
            key: tokens[1].trim().to_string(),
        })),
        _ => Err(io::Error::new(
            ErrorKind::InvalidData,
            "Too many tokens for get command",
        )),
    }
}

fn set_handler(tokens: &[&str]) -> io::Result<Command> {
    match tokens.len() {
        2 => {
            let pairs: Vec<&str> = tokens[1].split('=').collect();
            Ok(Command::Set(message::Set {
                key: pairs[0].to_string(),
                value: pairs[1].trim().to_string(),
            }))
        }
        _ => Err(io::Error::new(
            ErrorKind::InvalidData,
            "Too many tokens for get command",
        )),
    }
}

// fn backup_handler(tokens: &Vec<&str>) -> io::Result<Command> {
//     println!("{:?}", tokens);
//     match tokens.len() {
//         2 => {
//             let addr =
//                 SocketAddr::from_str(tokens[1].trim()).expect("TODO: Handle poorly formatted addr");
//             Ok(Command::InitiateBackup(message::InitiateBackup {
//                 addr: addr.to_string(),
//             }))
//         }
//         _ => Err(io::Error::new(
//             ErrorKind::InvalidData,
//             "Too many tokens for get command",
//         )),
//     }
// }
