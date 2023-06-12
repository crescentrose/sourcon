use std::{ops::RangeInclusive, str};

use crate::error::RconError;

/// PacketType enumerates the possible rcon packet types. They are seen as an
/// implementation detail of the library and while you can craft your own
/// packets, hopefully you will not have to.
#[derive(Debug)]
pub enum PacketType {
    /// Referred to as `SERVERDATA_AUTH` in Valve docs. This must be sent to the
    /// server prior to Exec commands.
    Auth,
    /// Referred to as `SERVERDATA_AUTH_RESPONSE` in Valve docs. This value is
    /// not actually checked by this library, we just kinda assume everything
    /// works fine.
    AuthResponse,
    /// Referred to as `SERVERDATA_EXECCOMMAND` in Valve docs. Use this for any
    /// generic command you may want to issue to the server.
    Exec,
    /// Referred to as `SERVERDATA_RESPONSE_VALUE` in Valve docs. This is the
    /// type that the responses should have, although that is not validated by
    /// this library.
    Response,
}

impl PacketType {
    /// Valve tells us that the fields in the header of a rcon packet are all
    /// signed 32-bit integers in low-endian, so we can easily convert like so.
    pub fn to_le_bytes(&self) -> [u8; 4] {
        let type_value: i32 = match self {
            PacketType::Auth => 3,
            PacketType::Exec => 2,
            PacketType::AuthResponse => 2, // not a bug, they do indeed have the same IDs...
            PacketType::Response => 0,
        };
        type_value.to_le_bytes()
    }
}

/// Convert an i32 into a [PacketType]. Since type 2 is ambiguous, we just kinda
/// sorta guess it will be a [PacketType::AuthResponse], as we don't expect the
/// server to send us an Exec command.
impl TryInto<PacketType> for i32 {
    type Error = RconError;

    fn try_into(self) -> Result<PacketType, Self::Error> {
        match self {
            3 => Ok(PacketType::Auth),
            2 => Ok(PacketType::AuthResponse),
            0 => Ok(PacketType::Response),
            n => Err(RconError::UnknownPacketType(n)),
        }
    }
}

/// According to the Valve wiki, rcon responses are split into 4kB packets.
pub type RawPacket = [u8; 4096];

/// Low level implementation of a rcon packet.
#[derive(Debug)]
pub struct Packet {
    id: i32,
    packet_type: PacketType,
    body: Option<String>,
}

impl Packet {
    /// From the docs:
    ///
    /// > Since the only one of these values that can change in
    /// > length is the body, an easy way to calculate the size of a packet is to
    /// > find the byte-length of the packet body, then add 10 to it.
    ///
    /// And that is exactly what we are going to do.
    pub const BASE_PACKET_SIZE: i32 = 10;

    const SIZE_RANGE: RangeInclusive<usize> = 0..=3;
    const ID_RANGE: RangeInclusive<usize> = 4..=7;
    const TYPE_RANGE: RangeInclusive<usize> = 8..=11;
    const BODY_OFFSET: usize = 12;

    /// Creates a new packet. `body` will likely become an [Option] in the
    /// future.
    pub fn new(id: i32, packet_type: PacketType, body: &str) -> Self {
        Packet {
            id,
            packet_type,
            body: Some(String::from(body)), // once told me
        }
    }

    /// Deserializes an incoming packet, splitting it up into headers and body.
    pub fn unpack(incoming: RawPacket) -> Result<Self, RconError> {
        // packet size = id (4) + type (4) + 2 (body + terminator)
        // -> body size = packet size - 10
        // -> offset = 12
        // -> last index = body size + offset
        // -> last index == 12? => no body

        let raw_size = &incoming[Self::SIZE_RANGE];
        let size = i32::from_le_bytes(raw_size.try_into()?);
        let body_size = size - Self::BASE_PACKET_SIZE;
        let last_elem: usize = body_size as usize + Self::BODY_OFFSET;

        let raw_id = &incoming[Self::ID_RANGE];
        let id = i32::from_le_bytes(raw_id.try_into()?);

        let raw_type = &incoming[Self::TYPE_RANGE];
        let packet_type: PacketType = i32::from_le_bytes(raw_type.try_into()?).try_into()?;

        let raw_body = &incoming[Self::BODY_OFFSET..];

        let body = if last_elem == Self::BODY_OFFSET {
            None
        } else {
            Some(str::from_utf8(&raw_body[..=last_elem])?.to_string())
        };

        let packet = Packet {
            id,
            packet_type,
            body,
        };

        Ok(packet)
    }

    /// Serializes a packet into an array of bytes.
    pub fn pack(&self) -> Vec<u8> {
        // packet structure: size, ID, type, body, terminator
        let mut payload = Vec::<u8>::new();
        payload.extend_from_slice(&self.size().to_le_bytes());
        payload.extend_from_slice(&self.id().to_le_bytes());
        payload.extend_from_slice(&self.packet_type().to_le_bytes());
        payload.extend_from_slice(self.body().unwrap_or(String::from("")).as_bytes());
        // null terminate the body (C++ interop 🤢), then null terminate the entire package
        payload.extend_from_slice(&[0, 0]);
        payload
    }

    /// Returns the size of the packet in bytes, excluding the size of the size
    /// field itself.
    pub fn size(&self) -> i32 {
        match self.body() {
            None => Self::BASE_PACKET_SIZE,
            Some(body) => body.len() as i32 + Self::BASE_PACKET_SIZE,
        }
    }

    /// A packet can have an optional ID that lets you track the responses to
    /// your commands. We implement this in the [crate::client::Client] to
    /// ensure that responses spanning multiple packets can be pieced back
    /// together on arrival.
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn packet_type(&self) -> &PacketType {
        &self.packet_type
    }

    pub fn body(&self) -> Option<String> {
        self.body.clone()
    }
}
