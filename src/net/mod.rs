pub mod client;
pub mod server;

use bytebuffer::ByteBuffer;
use mvutils::bytebuffer::ByteBufferExtras;
use mvutils::save::Savable;
use mvutils::Savable;
use std::{env, fs, io};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::SystemTime;
use log::{info, trace, warn};
use mvutils::utils::Time;

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
    trace!("Packet length: {len:?}");
    let len = u32::from_le_bytes(len);
    let mut buffer = vec![0u8; len as usize];
    trace!("Begin read packet of length {len}");
    stream.set_nonblocking(false).map_err(ReadPacketError::FromTcp)?;
    stream
        .read_exact(&mut buffer)
        .map_err(ReadPacketError::FromTcp)?;
    stream.set_nonblocking(true).map_err(ReadPacketError::FromTcp)?;
    trace!("End read packet of length {len}");

    let mut buffer = ByteBuffer::from_vec_le(buffer);

    P::load(&mut buffer).map_err(ReadPacketError::FromSavable)
}
