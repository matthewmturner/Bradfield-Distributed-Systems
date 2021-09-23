use std::io::{self, BufRead, BufReader, ErrorKind, Stdin, Write};
use std::net::TcpStream;

use prost::Message;

use super::super::ipc::message::request::Command;
use super::super::ipc::message::{self, request};

pub fn read_client_request(stdin: &mut Stdin) -> io::Result<String> {
    let mut reader = BufReader::new(stdin);

    let request = loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        break line;
    };
    Ok(request)
}

pub fn send_client_request(request: String, stream: &mut TcpStream) -> io::Result<()> {
    stream.write(request.as_bytes())?;
    Ok(())
}

pub fn send_protobuf<M>(message: M, stream: &mut TcpStream) -> io::Result<()>
where
    M: Message,
{
    let bytes = message.encode_length_delimited_to_vec();
    stream.write(&bytes)?;
    Ok(())
}

pub fn read_store_response(stream: &mut TcpStream) -> io::Result<String> {
    let mut reader = BufReader::new(stream);

    let response = loop {
        let mut bytes = String::new();
        reader.read_line(&mut bytes)?;
        break bytes;
    };
    Ok(response)
}

pub fn parse_request(input: String) -> io::Result<message::Request> {
    let tokens: Vec<&str> = input.split(" ").collect();
    let command = extract_command(tokens)?;
    Ok(message::Request {
        command: Some(command),
    })
}

fn extract_command(tokens: Vec<&str>) -> io::Result<Command> {
    let command = match tokens[0] {
        "get" | "Get" | "GET" => Ok(get_handler(&tokens)?),
        "set" | "Set" | "SET " => Ok(set_handler(&tokens)?),
        _ => Err(io::Error::new(ErrorKind::InvalidData, "Invalid command")),
    };
    command
}

fn get_handler(tokens: &Vec<&str>) -> io::Result<Command> {
    match tokens.len() {
        2 => Ok(Command::Get(message::Get {
            key: tokens[1].to_string(),
        })),
        _ => Err(io::Error::new(
            ErrorKind::InvalidData,
            "Too many tokens for get command",
        )),
    }
}

fn set_handler(tokens: &Vec<&str>) -> io::Result<Command> {
    match tokens.len() {
        2 => {
            let pairs: Vec<&str> = tokens[1].split("=").collect();
            Ok(Command::Set(message::Set {
                key: pairs[0].to_string(),
                value: pairs[1].to_string(),
            }))
        }
        _ => Err(io::Error::new(
            ErrorKind::InvalidData,
            "Too many tokens for get command",
        )),
    }
}
