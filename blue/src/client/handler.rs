use std::io::{self, BufRead, BufReader, Stdin, Write};
use std::net::TcpStream;

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
