use std::io::{self, Write};
use std::net::TcpStream;

use prost::Message;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream as asyncTcpStream;

pub async fn async_send_message<M>(message: M, stream: &mut asyncTcpStream) -> io::Result<()>
where
    M: Message,
{
    let length = message.encoded_len() as i32;
    println!("Sending message: {:?} \n\tOn Stream: {:?}", message, stream);
    let mut buf: Vec<u8> = Vec::with_capacity(length as usize);
    message.encode(&mut buf)?;
    stream.write_all(&length.to_le_bytes()).await?;
    stream.write_all(&buf).await?;
    Ok(())
}

pub fn send_message<M>(message: M, stream: &mut TcpStream) -> io::Result<()>
where
    M: Message,
{
    let length = message.encoded_len() as i32;
    println!("Sending message: {:?} \n\tOn Stream: {:?}", message, stream);
    let mut buf: Vec<u8> = Vec::with_capacity(length as usize);
    message.encode(&mut buf)?;
    stream.write_all(&length.to_le_bytes())?;
    stream.write_all(&buf)?;
    Ok(())
}
