use std::error::Error;
use std::io;
use std::io::Write;
use std::net::TcpStream;

use prost::Message;
use structopt::StructOpt;

extern crate blue;

use blue::client::args;
use blue::client::handler::{
    parse_request, read_client_request, read_store_response, send_client_request, send_pb_message,
};

fn main() -> Result<(), Box<dyn Error>> {
    let opt = args::Opt::from_args();
    let addr = format!("{}:{}", opt.host, opt.port);
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
        let pb = parse_request(user_request.clone())?;
        println!("{:?}", pb);
        send_pb_message(pb, &mut stream)?;
        // send_client_request(user_request, &mut stream)?;
        let response = read_store_response(&mut stream)?;
        println!("{}", response);
        input_num += 1;
    }
    Ok(())
}
