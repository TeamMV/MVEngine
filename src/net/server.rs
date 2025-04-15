use std::io::{ErrorKind, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::{io, thread};
use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::thread::JoinHandle;
use bytebuffer::ByteBuffer;
use crossbeam_channel::Sender;
use hashbrown::HashMap;
use log::{debug, error, info, warn};
use mvutils::hashers::U64IdentityHasher;
use mvutils::save::Savable;
use mvutils::unsafe_utils::DangerousCell;
use mvutils::utils;
use parking_lot::RwLock;
use crate::net::{try_read_packet, DisconnectReason, ReadPacketError};

pub type ClientId= u64;

pub struct Server<In: Savable, Out: Savable> {
    _maker: PhantomData<(In, Out)>,
    thread: Option<JoinHandle<()>>,
    stopper: Option<Sender<String>>,
    clients: Arc<RwLock<HashMap<ClientId, Arc<ClientEndpoint>, U64IdentityHasher>>>
}

impl<In: Savable, Out: Savable> Server<In, Out> {
    pub fn new() -> Self {
        Self {
            _maker: PhantomData::default(),
            thread: None,
            stopper: None,
            clients: Arc::new(RwLock::new(HashMap::with_hasher(U64IdentityHasher::default()))),
        }
    }

    pub fn listen<Handler: ServerHandler<In>>(&mut self, port: u16) {
        let (stop_sen, stop_rec) = crossbeam_channel::unbounded::<String>();
        let (disconnect_sen, disconnect_rec) = crossbeam_channel::unbounded::<(ClientId, DisconnectReason)>();
        let clients = self.clients.clone();
        let handle = thread::spawn(move || {
            let addr = SocketAddr::from(([127, 0, 0, 1], port));
            let socket = TcpListener::bind(addr);
            if let Err(e) = socket {
                error!("Could not start server: {e}");
                return;
            }
            let mut socket = socket.unwrap();
            if let Err(_) = socket.set_nonblocking(true) {
                error!("Cannot set TcpListener into non-blocking mode");
                return;
            }

            let handler = Handler::on_server_start(port);
            info!("Listening on port {port}");
            loop {
                if let Ok(stop_msg) = stop_rec.try_recv() {
                    info!("Server stopped with message: {stop_msg}");
                    handler.on_server_stop(&stop_msg);
                    return;
                }

                if let Ok((id, reason)) = disconnect_rec.try_recv() {
                    debug!("Attempt to disconnect client {id}, reason: {reason:?}");
                    let mut map = clients.write();
                    if let Some(client) = map.remove(&id) {
                        info!("Client {id} disconnected, reason: {reason:?}");
                        handler.on_client_disconnect(client, reason);
                    }
                }

                //check for incoming streams
                loop {
                    match socket.accept() {
                        Ok((stream, addr)) => {
                            info!("Client connected with address {addr:?}");
                            let id = utils::next_id("MVEngine::net::server::Server::listen");
                            let endpoint = ClientEndpoint::new(id, stream, disconnect_sen.clone());
                            let arc = Arc::new(endpoint);
                            handler.on_client_connect(arc.clone());
                            let mut map = clients.write();
                            map.insert(id, arc);
                        }
                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                            // No connection is ready to accept right now
                            break;
                        }
                        Err(e) => {
                            warn!("Error while accepting connection: {e}");
                            break;
                        }
                    }
                }
                //read streams into packets

                let mut map = clients.write();
                for endpoint in map.values_mut() {
                    let stream = endpoint.tcp.get_mut();
                    let mut packet = try_read_packet::<In>(stream);
                    while let Ok(inner) = packet {
                        handler.on_packet(endpoint.clone(), inner);
                        packet = try_read_packet::<In>(stream);
                    }
                    if let Some(e) = packet.err() {
                        if let ReadPacketError::FromTcp(tcp_err) = e {
                            match tcp_err.kind() {
                                ErrorKind::TimedOut => {
                                    if let Err(e) = disconnect_sen.send((endpoint.id, DisconnectReason::TimedOut)) {
                                        warn!("Error when attempting to send disconnect to server thread: {e}");
                                    }
                                }
                                ErrorKind::ConnectionReset
                                | ErrorKind::ConnectionAborted
                                | ErrorKind::UnexpectedEof
                                | ErrorKind::BrokenPipe
                                | ErrorKind::NotConnected => {
                                    if let Err(e) = disconnect_sen.send((endpoint.id, DisconnectReason::Disconnected)) {
                                        warn!("Error when attempting to send disconnect to server thread: {e}");
                                    }
                                }
                                _ => {}
                            }
                        } else if let ReadPacketError::FromSavable(s) = e {
                            warn!("Could not deserialize packet: {s}");
                        }
                    }
                }
            };
        });
        self.thread = Some(handle);
        self.stopper = Some(stop_sen);
    }

    pub fn stop(&mut self, message: &str) {
        if let Some(sender) = &mut self.stopper {
            if let Err(e) = sender.send(message.to_string()) {
                error!("Could not send stop message across stopper channel! {e}");
            }
        } else {
            warn!("Tried to stop server that is not started!");
        }
    }

    pub fn send_to_all_clients(&self, packet: Out) where Out: Clone {
        let map = self.clients.read();
        for client in map.values() {
            client.send(packet.clone());
        }
    }
}

pub trait ServerHandler<In: Savable> {
    fn on_server_start(port: u16) -> Self;
    fn on_client_connect(&self, client: Arc<ClientEndpoint>);
    fn on_client_disconnect(&self, client: Arc<ClientEndpoint>, reason: DisconnectReason);
    fn on_packet(&self, client: Arc<ClientEndpoint>, packet: In);
    fn on_server_stop(&self, message: &str);
}

pub struct ClientEndpoint {
    id: ClientId,
    tcp: DangerousCell<TcpStream>,
    disconnect_sender: Sender<(ClientId, DisconnectReason)>
}

impl ClientEndpoint {
    pub(crate) fn new(id: ClientId, stream: TcpStream, disconnect_sender: Sender<(ClientId, DisconnectReason)>) -> Self {
        Self {
            id,
            tcp: DangerousCell::new(stream),
            disconnect_sender,
        }
    }

    pub fn send<Out: Savable>(&self, packet: Out) {
        //build data (len_u32+bytes)
        let mut buffer = ByteBuffer::new();
        packet.save(&mut buffer);
        let len = buffer.len() as u32;
        let len_bytes: [u8; 4] = len.to_le_bytes();
        let mut vec = len_bytes.to_vec();
        vec.extend(buffer.into_vec());

        //write data
        let stream = self.tcp.get_mut();
        if let Err(e) = stream.write_all(&vec) {
            warn!("Error when attempting to write packet: {e}");
        }
    }

    pub fn disconnect(&self, reason: DisconnectReason) {
        if let Err(e) = self.disconnect_sender.send((self.id, reason)) {
            warn!("Error when attempting to send disconnect to server thread: {e}");
        }
        let stream = self.tcp.get_mut();
        if let Err(e) = stream.shutdown(Shutdown::Both) {
            warn!("Error when attempting to close socket connection: {e}");
        }
    }
}

unsafe impl Sync for ClientEndpoint {}