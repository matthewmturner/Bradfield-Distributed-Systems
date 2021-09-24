use std::error::Error;
use std::io::{BufRead, BufReader};
use std::net::TcpListener;

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:7777")?;

    for stream in listener.incoming() {
        match &stream {
            Ok(stream) => {
                let mut reader = BufReader::new(stream);
                let mut input = String::new();
                let byte_count = reader.read_line(&mut input)?;
                println!("Read {} bytes", byte_count);
                println!("{}", input)
            }
            Err(e) => println!("{}", e),
        }
    }
    Ok(())
}
