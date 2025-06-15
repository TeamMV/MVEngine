pub mod client;
pub mod server;

use bytebuffer::ByteBuffer;
use mvutils::bytebuffer::ByteBufferExtras;
use mvutils::save::Savable;
use mvutils::Savable;
use std::io;
use std::io::Read;
use std::net::TcpStream;
use log::{info, trace, warn};

#[derive(Clone, Savable, Debug)]
pub enum DisconnectReason {
    TimedOut,
    Disconnected,
}

pub(crate) enum ReadPacketError {
    FromTcp(io::Error),
    FromSavable(String),
}

pub(crate) fn try_read_packet<P: Savable>(stream: &mut TcpStream) -> Result<P, ReadPacketError> {
    let mut len = [0u8; 4];
    stream
        .read_exact(&mut len)
        .map_err(ReadPacketError::FromTcp)?;
    let len = u32::from_le_bytes(len);
    trace!("Packet length: {len:?}");
    let mut buffer = vec![0u8; len as usize];
    stream
        .read_exact(&mut buffer)
        .map_err(ReadPacketError::FromTcp)?;
    let mut buffer = ByteBuffer::from_vec_le(buffer);
    P::load(&mut buffer).map_err(ReadPacketError::FromSavable)
}
