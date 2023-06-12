use std::str;

#[derive(Debug)]
pub enum PacketType {
    // SERVERDATA_AUTH
    Auth,
    // SERVERDATA_EXECCOMMAND,
    Exec,
    // SERVERDATA_AUTH_RESPONSE
    AuthResponse,
    // SERVERDATA_RESPONSE_VALUE
    Response,
}

#[derive(Debug)]
pub enum Error {
    MalformedPackage,
}

impl PacketType {
    pub fn to_le_bytes(&self) -> [u8; 4] {
        let type_value: i32 = match self {
            PacketType::Auth => 3,
            PacketType::Exec => 2,
            PacketType::AuthResponse => 2,
            PacketType::Response => 0,
        };
        type_value.to_le_bytes()
    }
}

impl TryInto<PacketType> for i32 {
    type Error = Error;

    fn try_into(self) -> Result<PacketType, Self::Error> {
        match self {
            3 => Ok(PacketType::Auth),
            2 => Ok(PacketType::AuthResponse),
            0 => Ok(PacketType::Response),
            _ => Err(Error::MalformedPackage),
        }
    }
}

#[derive(Debug)]
pub struct Packet {
    id: i32,
    packet_type: PacketType,
    body: Option<String>,
}

pub type RawResponseBody = [u8; 4096];

impl Packet {
    pub const BASE_PACKAGE_SIZE: i32 = 10;

    pub fn new(id: i32, packet_type: PacketType, body: String) -> Self {
        Packet {
            id: id,
            packet_type: packet_type,
            body: Some(body),
        }
    }

    pub fn unpack(incoming: RawResponseBody) -> Result<Self, Error> {
        // packet size = id (4) + type (4) + 2 (body + terminator)
        // -> body size = packet size - 10
        // -> offset = 12
        // -> last index = body size + offset
        // -> last index == 12? => None

        let raw_size = &incoming[0..=3];
        let size = i32::from_le_bytes(raw_size.try_into().unwrap());
        let body_size = size - Self::BASE_PACKAGE_SIZE;
        let last_elem: usize = body_size as usize + 12;

        let raw_id = &incoming[4..=7];
        let id = i32::from_le_bytes(raw_id.try_into().unwrap());

        let raw_type = &incoming[8..=11];
        let packet_type: PacketType = i32::from_le_bytes(raw_type.try_into().unwrap())
            .try_into()
            .unwrap();

        let raw_body = &incoming[12..];

        let body = if last_elem == 12 {
            None
        } else {
            Some(str::from_utf8(&raw_body[..=last_elem]).unwrap().to_string())
        };

        let packet = Packet {
            id,
            packet_type,
            body,
        };

        Ok(packet)
    }

    pub fn pack(&self) -> Vec<u8> {
        // Size, ID, Type, Body, Terminator
        let mut payload = Vec::<u8>::new();
        payload.extend_from_slice(&self.size().to_le_bytes());
        payload.extend_from_slice(&self.id().to_le_bytes());
        payload.extend_from_slice(&self.packet_type().to_le_bytes());
        payload.extend_from_slice(&self.body().unwrap_or("").as_bytes());
        // null terminate the body (C++ interop 🤢), then null terminate the entire package
        payload.extend_from_slice(&[0 as u8, 0 as u8]);
        payload
    }

    // Since the only one of these values that can change in length is the body,
    // an easy way to calculate the size of a packet is to find the byte-length
    // of the packet body, then add 10 to it.
    pub fn size(&self) -> i32 {
        match self.body() {
            None => Self::BASE_PACKAGE_SIZE as i32,
            Some(body) => body.len() as i32 + Self::BASE_PACKAGE_SIZE,
        }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn packet_type(&self) -> &PacketType {
        &self.packet_type
    }

    pub fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }
}
