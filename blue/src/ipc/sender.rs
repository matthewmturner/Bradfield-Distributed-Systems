use std::io::{self, Write};
use std::net::TcpStream;

use prost::Message;

pub fn send_message<M>(message: M, stream: &mut TcpStream) -> io::Result<()>
where
    M: Message,
{
    let length = message.encoded_len() as i32;
    let mut buf: Vec<u8> = Vec::with_capacity(length as usize);
    message.encode(&mut buf)?;
    stream.write_all(&length.to_le_bytes())?;
    stream.write_all(&buf)?;
    Ok(())
}
