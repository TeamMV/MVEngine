pub mod client;
pub mod middleware;

use std::io::Write;
use std::marker::PhantomData;
use std::net::{Shutdown, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::{Arc};
use bytebuffer::ByteBuffer;
use hashbrown::HashMap;
use log::warn;
use mvutils::hashers::U64IdentityHasher;
use mvutils::save::Savable;
use mvutils::unsafe_utils::DangerousCell;
use parking_lot::Mutex;
use crate::net::sealed::ConnectionType;

mod sealed {
    pub trait ConnectionType {}
}

pub struct Server;
pub struct Client;

impl ConnectionType for Server {}
impl ConnectionType for Client {}

pub type ClientId = u64;

pub struct ConnectionHandler<In: Savable, Out: Savable, Type: ConnectionType, Handler: PacketHandler<In>> {
    pub(crate) handler: Handler,
    _phantom: PhantomData<(In, Out, Type)>,

    endpoints: Option<Arc<Mutex<HashMap<u64, ClientEndpoint, U64IdentityHasher>>>>,
    connection: Option<Arc<DangerousCell<TcpStream>>>,
}

unsafe impl<In: Savable, Out: Savable, Type: ConnectionType, Handler: PacketHandler<In>> Send for ConnectionHandler<In, Out, Type, Handler> {}
unsafe impl<In: Savable, Out: Savable, Type: ConnectionType, Handler: PacketHandler<In>> Sync for ConnectionHandler<In, Out, Type, Handler> {}

pub enum DisconnectReason {
    Disconnected,
    TimedOut,
    Kicked,
}

pub trait PacketHandler<In: Savable>: Sized {
    /// This event is never fired on a Client ConnectionHandler
    fn connection<Out: Savable, Type: ConnectionType>(&self, connection_handler: &ConnectionHandler<In, Out, Type, Self>, id: ClientId);

    fn disconnection<Out: Savable, Type: ConnectionType>(&self, connection_handler: &ConnectionHandler<In, Out, Type, Self>, id: ClientId, reason: DisconnectReason);

    fn incoming<Out: Savable, Type: ConnectionType>(&self, connection_handler: &ConnectionHandler<In, Out, Type, Self>, id: ClientId, packet: In);
}

impl<In: Savable + 'static, Out: Savable + 'static, Handler: PacketHandler<In> + 'static> ConnectionHandler<In, Out, Server, Handler> {
    pub fn listen(port: u16, handler: Handler) -> Arc<Self> {
        let listener = TcpListener::bind(("127.0.0.1", port)).expect("Couldn't startup server!");
        let this = Arc::new(Self {
            handler,
            _phantom: PhantomData::default(),
            endpoints: Some(Arc::new(Mutex::new(HashMap::with_hasher(U64IdentityHasher::default())))),
            connection: None,
        });

        let this2 = this.clone();
        std::thread::spawn(move || {
            loop {
                if let Ok((socket, _)) = listener.accept() {
                    let endpoint = ClientEndpoint::new(socket, this2.clone());
                    let id = endpoint.id;
                    let map = this2.endpoints.clone().unwrap();
                    map.lock().insert(endpoint.id, endpoint);
                    this2.handler.connection(&*this2, id);
                }
            }
        });
        this
    }

    pub fn get_client_endpoint(&self, id: ClientId) -> Option<ClientEndpoint> {
        self.endpoints.clone().unwrap().lock().get(&id).cloned()
    }

    pub fn pop_client_endpoint(&self, id: ClientId) -> Option<ClientEndpoint> {
        self.endpoints.clone().unwrap().lock().remove(&id)
    }

    pub fn send_all(&self, out: Out) {
        let mut buffer = ByteBuffer::new();
        out.save(&mut buffer);
        let bytes = middleware::encode(buffer.into_vec());

        let map = self.endpoints.clone().unwrap();
        for endpoint in map.lock().values() {
            self.send_raw(endpoint.socket.get_mut(), &bytes);
        }
    }

    pub fn send(&self, id: ClientId, out: Out) {
        if let Some(endpoint) = self.get_client_endpoint(id) {
            let mut buffer = ByteBuffer::new();
            out.save(&mut buffer);
            let bytes = middleware::encode(buffer.into_vec());
            self.send_raw(endpoint.socket.get_mut(), &bytes);
        }
    }

    pub fn disconnect_all(&self) {
        let map = self.endpoints.clone().unwrap();

        for endpoint in map.lock().values() {
            if let Err(_) = endpoint.socket.get().shutdown(Shutdown::Both) {
                warn!("Couldn't shutdown connection with {}", endpoint.addr);
            }
        }
    }

    pub fn disconnect(&self, id: ClientId, reason: DisconnectReason) {
        if let Some(endpoint) = self.pop_client_endpoint(id) {
            if let Err(_) = endpoint.socket.get().shutdown(Shutdown::Both) {
                warn!("Couldn't shutdown connection with {}", endpoint.addr);
            }
            self.handler.disconnection(self, id, reason);
        }
    }
}

impl<In: Savable, Out: Savable, Handler: PacketHandler<In>> ConnectionHandler<In, Out, Client, Handler> {
    pub fn connect(address: impl ToSocketAddrs, handler: Handler) -> std::io::Result<Arc<Self>> {
        let this = Arc::new(Self {
            handler,
            _phantom: PhantomData::default(),
            endpoints: None,
            connection: Some(Arc::new(DangerousCell::new(TcpStream::connect(address)?))),
        });
        Ok(this)
    }

    pub fn send(&self, out: Out) {
        let mut buffer = ByteBuffer::new();
        out.save(&mut buffer);
        let tcp = self.connection.clone().unwrap();
        self.send_raw(tcp.get_mut(), &middleware::encode(buffer.into_vec()));
    }

    pub fn disconnect(self) {
        let tcp = self.connection.clone().unwrap();
        if let Err(_) = tcp.get_mut().shutdown(Shutdown::Both) {
            warn!("Couldn't shutdown connection");
        }
    }
}

impl<In: Savable, Out: Savable, Type: ConnectionType, Handler: PacketHandler<In>> ConnectionHandler<In, Out, Type, Handler> {
    fn send_raw(&self, socket: &mut TcpStream, data: &[u8]) {
        let addr = socket.peer_addr().map(|a| a.to_string()).unwrap_or("<invalid address>".to_string());
        if let Err(_) = socket.write_all(data) {
            warn!("Data could not be written to {addr}");
        }
    }
}