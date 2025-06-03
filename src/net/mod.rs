pub mod server;
pub mod client;

use std::io;
use bytebuffer::{ByteBuffer, Endian};
use mvutils::save::Savable;
use mvutils::Savable;
use std::io::Read;
use std::net::TcpStream;
use log::debug;
use mvutils::bytebuffer::ByteBufferExtras;

#[derive(Clone, Savable, Debug)]
pub enum DisconnectReason {
    TimedOut,
    Disconnected,
}

pub(crate) enum ReadPacketError {
    FromTcp(io::Error),
    FromSavable(String)
}

pub(crate) fn try_read_packet<P: Savable>(stream: &mut TcpStream) -> Result<P, ReadPacketError> {
    let mut len = [08; 4];
    stream.read_exact(&mut len).map_err(ReadPacketError::FromTcp)?;
    let len = u32::from_be_bytes(len);
    let mut buffer = vec![0u8; len as usize];
    stream.read_exact(&mut buffer).map_err(ReadPacketError::FromTcp)?;
    let mut buffer = ByteBuffer::from_vec_le(buffer);
    P::load(&mut buffer).map_err(ReadPacketError::FromSavable)
}