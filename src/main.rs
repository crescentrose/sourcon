use std::str;
use std::{error::Error, io};

use sourcon::packet::{Packet, PacketType};
use tokio::net::TcpStream;
mod packet;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let host = "dev.viora.sh:27016";
    let stream = TcpStream::connect(host).await?;

    let auth = Packet::new(1, PacketType::Auth, "poopxd".to_string());
    let status = Packet::new(2, PacketType::Exec, "status".to_string());

    let mut buf = [0; 4096];

    stream.writable().await?;
    stream.try_write(&auth.pack()).unwrap();

    println!("response: {:?}", read_stream(&stream).await?);
    println!("response: {:?}", read_stream(&stream).await?);

    stream.writable().await?;
    stream.try_write(&status.pack()).unwrap();

    let status_res = read_stream(&stream).await?;
    println!("{}", status_res.body().unwrap());

    Ok(())
}

async fn read_stream(stream: &TcpStream) -> Result<Packet, Box<dyn Error>> {
    let mut buf = [0; 4096];
    loop {
        stream.readable().await?;
        match stream.try_read(&mut buf) {
            Ok(_) => break,
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
            Err(e) => panic!("{:?}", e),
        }
    }
    Ok(Packet::unpack(buf).unwrap())
}
