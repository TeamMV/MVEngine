pub mod client;
pub mod server;

use bytebuffer::ByteBuffer;
use log::{info, trace, warn};
use mvutils::Savable;
use mvutils::bytebuffer::ByteBufferExtras;
use mvutils::save::Savable;
use mvutils::utils::Time;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::SystemTime;
use std::{env, fs, io};

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
    let mut buffer = vec![0u8; len as usize];
    stream.set_nonblocking(false).unwrap();
    stream
        .read_exact(&mut buffer)
        .map_err(ReadPacketError::FromTcp)?;
    stream.set_nonblocking(true).unwrap();

    let mut buffer = ByteBuffer::from_vec_le(buffer);

    P::load(&mut buffer).map_err(ReadPacketError::FromSavable)
}
