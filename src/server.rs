use std::net::SocketAddr;

use log::{error, info};
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
    task::JoinHandle,
};

use crate::{
    error::RconError,
    packet::{self, Packet},
};

pub struct Server {
    listener: TcpListener,
    password: String,
}

impl Server {
    pub async fn start<F>(handler: F) -> Result<JoinHandle<()>, RconError>
    where
        F: Fn(Result<Packet, RconError>) + Send + Sync + Copy + 'static,
    {
        let test_packet = packet::Packet::new(1, packet::PacketType::Exec, "hello world");
        info!("try this sample packet: {:x?}", test_packet.pack());

        let addr = "127.0.0.1:27015";
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(RconError::BindError)?;

        let handle: JoinHandle<()> = tokio::spawn(async move {
            info!("server running on {}", addr);
            loop {
                let conn = listener.accept().await;
                match conn {
                    Ok((stream, addr)) => {
                        tokio::spawn(async move { handler(Server::process(stream, addr).await) });
                    }
                    Err(e) => error!("{:?}", e),
                }
            }
        });

        Ok(handle)
    }

    async fn process(mut stream: TcpStream, addr: SocketAddr) -> Result<Packet, RconError> {
        info!("accept from {:?}", addr);

        let mut buf: [u8; 4096] = [0; 4096];
        stream
            .read(&mut buf)
            .await
            .map_err(RconError::ReceiveError)?;

        packet::Packet::unpack(buf)
    }
}
