use std::io::{self, BufRead, BufReader, Stdin, Write};
use std::net::TcpStream;

use prost::Message;

use super::super::ipc::message;
use super::super::ipc::message::request;

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

pub fn read_store_response(stream: &mut TcpStream) -> io::Result<String> {
    let mut reader = BufReader::new(stream);

    let response = loop {
        let mut bytes = String::new();
        reader.read_line(&mut bytes)?;
        break bytes;
    };
    Ok(response)
}

pub fn extract_client_request_type(request: String) -> io::Result<request::Type> {
    let tokens: Vec<&str> = request.split(" ").collect();

    let request_type: io::Result<message::Request> = match tokens[0] {
        "get" | "Get" | "GET" => Ok(message::Request {
            command: message::Get {
                key: tokens[1].to_string(),
            },
        }),
        "set" | "Set" | "SET" => {
            let set_tokens: Vec<&str> = tokens[1].split("=").collect();
            Ok(message::Request {
                command: message::Set {
                    key: set_tokens[0],
                    value: set_tokens[1],
                },
            })
        }
        _ => Err(()),
    };
}
