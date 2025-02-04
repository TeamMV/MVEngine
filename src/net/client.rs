use std::io::Read;
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;
use bytebuffer::ByteBuffer;
use log::{info, warn};
use mvutils::save::Savable;
use mvutils::unsafe_utils::DangerousCell;
use crate::net::{middleware, ConnectionHandler, DisconnectReason, PacketHandler};

#[derive(Clone)]
pub struct ClientEndpoint {
    pub(crate) id: u64,
    pub(crate) socket: Arc<DangerousCell<TcpStream>>,
    pub(crate) addr: String,
}

impl ClientEndpoint {
    pub(crate) fn new<In: Savable + 'static, Out: Savable + 'static, Handler: PacketHandler<In> + 'static>(socket: TcpStream, connection_handler: Arc<ConnectionHandler<In, Out, Server, Handler>>) -> Self {
        let addr = socket.peer_addr().map(|a| a.to_string()).unwrap_or("<invalid address>".to_string());
        info!("Incoming connection from {addr}");
        let this = Self {
            id: mvutils::utils::next_id("MVEngine::Network::client_endpoint"),
            socket: Arc::new(DangerousCell::new(socket)),
            addr,
        };
        let _ = this.socket.get_mut().set_read_timeout(Some(Duration::from_secs(1)));
        let _ = this.socket.get_mut().set_write_timeout(Some(Duration::from_secs(1)));

        let this2 = this.clone();
        std::thread::spawn(move || {
            let socket = this2.get_socket();
            loop {
                let mut len_buffer = [0u8; 4];
                if let Err(_) = socket.get_mut().read_exact(len_buffer.as_mut()) {
                    warn!("Failed to read from {}", this2.addr);
                    connection_handler.disconnect(this2.id, DisconnectReason::TimedOut);
                    break;
                } else {
                    let len = u32::from_le_bytes(len_buffer);
                    let mut buffer = vec![0u8; len as usize];
                    if let Err(_) = socket.get_mut().read_exact(buffer.as_mut()) {
                        warn!("Failed to read from {}", this2.addr);
                        connection_handler.disconnect(this2.id, DisconnectReason::TimedOut);
                        break;
                    } else {
                        let buffer = middleware::decode(buffer);
                        let mut bytebuffer = ByteBuffer::from_bytes(buffer.as_slice());
                        let packet = In::load(&mut bytebuffer);
                        match packet {
                            Ok(in_packet) => {
                                connection_handler.handler.incoming(&*connection_handler, this2.id, in_packet);
                            }
                            Err(error) => {
                                warn!("Malformed packet: {error}");
                            }
                        }
                    }
                }
            }
        });
        this
    }

    fn get_socket(&self) -> Arc<DangerousCell<TcpStream>> {
        self.socket.clone()
    }
}

unsafe impl Send for ClientEndpoint {}
unsafe impl Sync for ClientEndpoint {}
