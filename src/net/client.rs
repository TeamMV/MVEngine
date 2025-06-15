use crate::net::{try_read_packet, DisconnectReason, ReadPacketError};
use bytebuffer::{ByteBuffer, Endian};
use crossbeam_channel::Sender;
use log::{debug, error, info, warn};
use mvutils::save::Savable;
use parking_lot::RwLock;
use std::io::{ErrorKind, Write};
use std::marker::PhantomData;
use std::net::{Shutdown, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

pub struct Client<In: Savable, Out: Savable> {
    _maker: PhantomData<In>,
    _thread: JoinHandle<()>,
    disconnect_sender: Sender<DisconnectReason>,
    packet_sender: Sender<Out>,
}

impl<In: Savable, Out: Savable + Send + 'static> Client<In, Out> {
    pub fn connect<Handler: ClientHandler<In> + Sync + 'static>(
        to: impl ToSocketAddrs,
        handler: Arc<RwLock<Handler>>,
    ) -> Option<Self> {
        let tcp = TcpStream::connect(to);
        if let Err(e) = tcp {
            error!("Could not connect to server, {e}");
            return None;
        }
        let mut tcp = tcp.unwrap();
        if let Err(_) = tcp.set_nonblocking(true) {
            error!("Cannot set TcpStream into non-blocking mode");
            return None;
        }
        info!("Connected to server");

        let (disconnect_sen, disconnect_rec) = crossbeam_channel::unbounded();
        let (packet_sen, packet_rec) = crossbeam_channel::unbounded::<Out>();

        let cloned_dis_sen = disconnect_sen.clone();
        let cloned = handler.clone();

        let handle = thread::spawn(move || {
            fn write_packet(tcp: &mut TcpStream, vec: &[u8]) {
                let mut written = 0;

                while written < vec.len() {
                    match tcp.write(&vec[written..]) {
                        Ok(0) => {
                            warn!("Socket closed while writing");
                            break;
                        }
                        Ok(n) => written += n,
                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                            debug!("WouldBlock error, attempting to rewrite packet");
                            thread::sleep(Duration::from_millis(1));
                            continue;
                        }
                        Err(e) => {
                            warn!("Error when attempting to write packet: {e}");
                            break;
                        }
                    }
                }
            }

            let mut handler = cloned.write();
            handler.on_connected();
            drop(handler);
            loop {
                handler = cloned.write();
                if let Ok(reason) = disconnect_rec.try_recv() {
                    debug!("Disconnecting from server, reason: {reason:?}");
                    if let Err(e) = tcp.shutdown(Shutdown::Both) {
                        warn!("Error when shutting down socket connection: {e}");
                    }
                    return;
                }

                while let Ok(packet) = packet_rec.try_recv() {
                    // Build data (len_u32 + bytes)
                    let mut buffer = ByteBuffer::new();
                    buffer.set_endian(Endian::LittleEndian);
                    packet.save(&mut buffer);
                    let len = buffer.len() as u32;
                    let len_bytes: [u8; 4] = len.to_le_bytes();
                    let mut vec = len_bytes.to_vec();
                    vec.extend(buffer.into_vec());

                    // Write data
                    write_packet(&mut tcp, &vec);
                }

                let mut packet = try_read_packet::<In>(&mut tcp);
                while let Ok(inner) = packet {
                    handler.on_packet(inner);
                    packet = try_read_packet::<In>(&mut tcp);
                }
                if let Some(e) = packet.err() {
                    if let ReadPacketError::FromTcp(tcp_err) = e {
                        match tcp_err.kind() {
                            ErrorKind::TimedOut => {
                                if let Err(e) = disconnect_sen.send(DisconnectReason::TimedOut) {
                                    warn!("Error when attempting to send disconnect to server thread: {e}");
                                }
                            }
                            ErrorKind::ConnectionReset
                            | ErrorKind::ConnectionAborted
                            | ErrorKind::UnexpectedEof
                            | ErrorKind::BrokenPipe
                            | ErrorKind::NotConnected => {
                                if let Err(e) = disconnect_sen.send(DisconnectReason::Disconnected)
                                {
                                    warn!("Error when attempting to send disconnect to server thread: {e}");
                                }
                            }
                            _ => {
                                //WouldBlock
                                debug!("WouldBlock on read packet");
                            }
                        }
                    } else {
                        if let ReadPacketError::FromSavable(s) = e {
                            println!("savable error: {s}");
                        }
                    }
                }

                drop(handler);
                thread::sleep(Duration::from_millis(10));
            }
        });

        Some(Self {
            _maker: PhantomData::default(),
            _thread: handle,
            disconnect_sender: cloned_dis_sen,
            packet_sender: packet_sen,
        })
    }

    pub fn disconnect(&mut self, reason: DisconnectReason) {
        if let Err(e) = self.disconnect_sender.send(reason) {
            warn!("Error when attempting to send disconnect to server thread: {e}");
        }
    }

    pub fn send(&mut self, packet: Out) {
        if let Err(e) = self.packet_sender.send(packet) {
            warn!("Error when sending packet: {e}");
        }
    }
}

pub trait ClientHandler<In: Savable>: Send {
    fn on_connected(&mut self);
    fn on_disconnected(&mut self, reason: DisconnectReason);
    fn on_packet(&mut self, packet: In);
}
