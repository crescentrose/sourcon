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

pub struct Packet {
    id: i32,
    packet_type: PacketType,
    body: String,
}

impl Packet {
    pub const BASE_PACKAGE_SIZE: i32 = 10;

    pub fn new(id: i32, packet_type: PacketType, body: String) -> Self {
        Packet {
            id: id,
            packet_type: packet_type,
            body: body,
        }
    }

    pub fn unpack(incoming: Vec<u8>) -> Result<Self, Error> {
        todo!()
    }

    // Since the only one of these values that can change in length is the body,
    // an easy way to calculate the size of a packet is to find the byte-length
    // of the packet body, then add 10 to it.
    pub fn size(&self) -> i32 {
        self.body.len() as i32 + Self::BASE_PACKAGE_SIZE
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn packet_type(&self) -> &PacketType {
        &self.packet_type
    }

    pub fn body(&self) -> &str {
        self.body.as_ref()
    }

    pub fn pack(&self) -> Vec<u8> {
        // Size, ID, Type, Body, Terminator
        let mut payload = Vec::<u8>::new();
        payload.extend_from_slice(&self.size().to_le_bytes());
        payload.extend_from_slice(&self.id().to_le_bytes());
        payload.extend_from_slice(&self.packet_type().to_le_bytes());
        payload.extend_from_slice(&self.body().as_bytes());
        // null terminate the body (C++ interop 🤢), then null terminate the entire package
        payload.extend_from_slice(&[0 as u8, 0 as u8]);
        payload
    }
}
