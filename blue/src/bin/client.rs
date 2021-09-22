use std::error::Error;
use std::io;
use std::net::TcpStream;
use std::{env, io::Write};

extern crate blue;

use blue::client::handler::{read_client_request, read_store_response, send_client_request};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let port = &args[1];
    let addr = format!("127.0.0.1:{}", port);
    let mut stream = TcpStream::connect(addr)?;
    let response = read_store_response(&mut stream)?;
    print!("{}", response);
    io::stdout().flush()?;

    let mut input_num: i32 = 1;

    let mut stdin = io::stdin();

    loop {
        let msg = format!("[{}] ", input_num);
        print!("{}", msg);
        io::stdout().flush()?;
        let user_request = read_client_request(&mut stdin)?;
        send_client_request(user_request, &mut stream)?;
        let response = read_store_response(&mut stream)?;
        println!("{}", response);
        input_num += 1;
    }
    Ok(())
}
