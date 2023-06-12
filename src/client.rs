use crate::{
    error::RconError,
    packet::{Packet, PacketType},
};
use log::trace;
use tokio::net::TcpStream;

/// Simple asynchronous rcon client. Call `connect()` to establish a connection
/// and authenticate. The client should be `mut` as it keeps a counter used for
/// [Packet] IDs.
///
/// ## Example
/// ```no_run
/// use sourcon::client::Client;
/// use std::error::Error;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn Error>> {
///     let host = "dev.viora.sh:27016";
///     // client must be mutable so we can increment packet IDs
///     let mut client = Client::connect(host, "<put rcon password here>").await?;
///     let response = client.command("echo hi").await?;
///
///     assert_eq!(response.body(), "hi");
///     Ok(())
/// }
/// ```
pub struct Client {
    next_packet_id: i32,
    stream: TcpStream,
}

/// Container struct for a response that can be glued together from multiple [Packet]s.
pub struct Response {
    body: String,
}

impl Response {
    pub fn body(&self) -> &str {
        self.body.as_ref()
    }
}

impl Client {
    pub async fn connect(host: &str, password: &str) -> Result<Self, RconError> {
        let stream = TcpStream::connect(host)
            .await
            .map_err(RconError::UnreachableHost)?;

        trace!("opened tcp stream to {}, attempting auth", host);

        Self::auth(password, &stream).await?;

        trace!("auth complete");

        Ok(Client {
            next_packet_id: 100, // IDs 1-99 are reserved for auth (even though we realistically only need two)
            stream,
        })
    }

    /// Run a rcon command asynchronously. In case of a response being split
    /// between multiple packets, they will be joined together afterwards.
    pub async fn command(&mut self, command: &str) -> Result<Response, RconError> {
        let command_packet = self.create_packet(command);
        // since srcds can split up the response but it won't tell us how many
        // packets to expect, we send a second packet immediately afterwards
        // with a blank command so that we can get a confirmation that there are
        // no more packets in response to our command.
        let tracking_packet = self.create_packet("");

        trace!("sending main packet to server");
        Self::write_to_stream(&command_packet, &self.stream).await?;
        trace!("sending tracking (blank) packet to server");
        Self::write_to_stream(&tracking_packet, &self.stream).await?;

        let mut responses = Vec::<Packet>::new();

        loop {
            // we are guaranteed to receive responses to packets in the order we sent them
            // so let's collect responses until we receive the ID for the tracking packet
            let response = Self::read_from_stream(&self.stream).await?;
            trace!("receive response for packet id {}", response.id());
            if response.id() == tracking_packet.id() {
                trace!("that was the tracking packet, completing response");
                break;
            } else {
                responses.push(response);
            }
        }

        let response: String = responses
            .iter()
            .map(|packet| packet.body().unwrap_or(String::from("")))
            .collect();

        Ok(Response { body: response })
    }

    fn create_packet(&mut self, command: &str) -> Packet {
        self.next_packet_id += 1;

        Packet::new(self.next_packet_id, PacketType::Exec, command)
    }

    /// Special case of `command` that will probably be generalized later.
    async fn auth(password: &str, stream: &TcpStream) -> Result<(), RconError> {
        let auth_packet = Packet::new(1, PacketType::Auth, password);
        let tracking_packet = Packet::new(2, PacketType::Exec, "");

        trace!("sending auth packet to server");
        Self::write_to_stream(&auth_packet, stream).await?;
        trace!("sending tracking (blank) packet to server for auth");
        Self::write_to_stream(&tracking_packet, stream).await?;

        loop {
            let response = Self::read_from_stream(stream).await?;
            trace!("receive response for packet id {}", response.id());
            if response.id() == -1 {
                return Err(RconError::AuthenticationError);
            }

            if response.id() == tracking_packet.id() {
                trace!("that was the tracking packet, completing auth");
                break;
            }
        }
        Ok(())
    }

    async fn write_to_stream(packet: &Packet, stream: &TcpStream) -> Result<(), RconError> {
        loop {
            stream.writable().await.map_err(RconError::SendError)?;

            match stream.try_write(&packet.pack()) {
                Ok(_) => return Ok(()),
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
                Err(e) => return Err(RconError::SendError(e)),
            }
        }
    }

    async fn read_from_stream(stream: &TcpStream) -> Result<Packet, RconError> {
        let mut buf = [0; 4096];

        loop {
            stream.readable().await.map_err(RconError::ReceiveError)?;
            match stream.try_read(&mut buf) {
                Ok(_) => break,
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
                Err(e) => return Err(RconError::ReceiveError(e)),
            }
        }

        Packet::unpack(buf)
    }
}
