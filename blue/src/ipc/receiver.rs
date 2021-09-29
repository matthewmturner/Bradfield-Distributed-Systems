use std::io::{self, Read};
use std::net::TcpStream;

use prost::Message;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream as asyncTcpStream;

pub async fn async_read_message<M: Message + Default>(
    stream: &mut asyncTcpStream,
) -> io::Result<M> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = i32::from_le_bytes(len_buf);
    println!("Incoming message length: {}", len);
    let mut buf = vec![0u8; len as usize];
    stream.read_exact(&mut buf).await?;
    println!("Reading buf: {:?}", buf);
    let user_input = M::decode(&mut buf.as_slice())?;
    Ok(user_input)
}

pub fn read_message<M: Message + Default>(stream: &mut TcpStream) -> io::Result<M> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let len = i32::from_le_bytes(len_buf);
    println!("Incoming message length: {}", len);
    let mut buf = vec![0u8; len as usize];
    stream.read_exact(&mut buf)?;
    println!("Reading buf: {:?}", buf);
    let user_input = M::decode(&mut buf.as_slice())?;
    Ok(user_input)
}
