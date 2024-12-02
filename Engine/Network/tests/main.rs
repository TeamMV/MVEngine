use mvutils::Savable;

#[derive(Savable)]
pub enum ServerboundPacket {
    Connect(ServerboundConnectPacket),
    Disconnect,
}

#[derive(Savable)]
pub enum ClientboundPacket {
    ClientConnected(ClientboundClientConnectPacket),
    ClientDisconnected(ClientboundClientDisconnectPacket),
}

#[derive(Savable)]
pub struct ServerboundConnectPacket {

}

#[derive(Savable)]
pub struct ClientboundClientConnectPacket {

}

#[derive(Savable)]
pub struct ClientboundClientDisconnectPacket {

}

fn main() {

}