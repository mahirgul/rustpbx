use std::net::SocketAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportProtocol {
    Udp,
    Tcp,
    Tls,
    Ws,
    Wss,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SipAddr {
    pub addr: SocketAddr,
    pub protocol: TransportProtocol,
}

impl SipAddr {
    pub fn new(addr: SocketAddr, protocol: TransportProtocol) -> Self {
        SipAddr { addr, protocol }
    }
}
