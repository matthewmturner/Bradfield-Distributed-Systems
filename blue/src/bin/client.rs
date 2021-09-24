use std::error::Error;
use std::io;
use std::io::Write;
use std::net::TcpStream;

use structopt::StructOpt;

extern crate blue;

use blue::client::args;
use blue::client::handler::{parse_request, read_client_request};
use blue::ipc::message;
use blue::ipc::receiver::read_message;
use blue::ipc::sender::send_message;

fn main() -> Result<(), Box<dyn Error>> {
    let opt = args::Opt::from_args();
    let addr = format!("{}:{}", opt.host, opt.port);
    let mut stream = TcpStream::connect(addr)?;
    let welcome = read_message::<message::Welcome>(&mut stream)?;
    print!("{}", welcome.message);
    io::stdout().flush()?;

    let mut input_num: i32 = 1;

    let mut stdin = io::stdin();

    loop {
        let msg = format!("[{}] ", input_num);
        print!("{}", msg);
        io::stdout().flush()?;
        let user_request = read_client_request(&mut stdin)?;
        let pb = parse_request(user_request.clone())?;
        send_message(pb, &mut stream)?;
        let response = read_message::<message::Response>(&mut stream)?;
        println!("{:?}", response);
        input_num += 1;
    }
    Ok(())
}
