use thiserror::Error;

#[derive(Error, Debug)]
pub enum RconError {
    #[error("unknown rcon packet type: {0}")]
    UnknownPacketType(i32),
    #[error("packet header malformed (can't parse size, id or type)")]
    MalformedPacketHeader(#[from] std::array::TryFromSliceError),
    #[error("packet body malformed (not valid ascii or utf-8)")]
    MalformedPacketBody(#[from] std::str::Utf8Error),
    #[error("host cannot be reached")]
    UnreachableHost(#[source] std::io::Error),
    #[error("cannot send message to host")]
    SendError(#[source] std::io::Error),
    #[error("cannot receive response from host")]
    ReceiveError(#[source] std::io::Error),
}
