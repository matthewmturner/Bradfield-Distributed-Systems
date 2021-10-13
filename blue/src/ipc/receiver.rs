use std::io::{self, Read};
use std::net::TcpStream;

use bytes::BytesMut;
use log::debug;

use prost::{decode_length_delimiter, Message};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream as asyncTcpStream;

pub async fn async_read_message<M: Message + Default>(
    stream: &mut asyncTcpStream,
) -> io::Result<M> {
    let mut len_buf = [0u8; 4];
    debug!("Reading message length");
    stream.read_exact(&mut len_buf).await?;
    let len = i32::from_le_bytes(len_buf);
    let mut buf = vec![0u8; len as usize];
    debug!("Reading message");
    stream.read_exact(&mut buf).await?;
    let user_input = M::decode(&mut buf.as_slice())?;
    debug!("Received message: {:?}", user_input);
    Ok(user_input)
}

pub fn read_message<M: Message + Default>(stream: &mut TcpStream) -> io::Result<M> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let len = i32::from_le_bytes(len_buf);
    let mut buf = vec![0u8; len as usize];
    stream.read_exact(&mut buf)?;
    let user_input = M::decode(&mut buf.as_slice())?;
    debug!("Received message: {:?}", user_input);
    Ok(user_input)
}

pub fn arm(stream: &mut TcpStream) -> io::Result<()> {
    let mut buf = BytesMut::with_capacity(10);
    let read_bytes = stream.peek(&mut buf)?;
    debug!("Read {} bytes", read_bytes);
    let length = decode_length_delimiter(buf)?;
    debug!("Length: {}", length);
    Ok(())
}
