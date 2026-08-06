use sipcore::types::message::SipMessage;
use std::net::SocketAddr;
use tracing::debug;

pub struct NatTraversal;

impl NatTraversal {
    #[allow(dead_code)]
    pub fn process_inbound(&self, _msg: &mut SipMessage, src_addr: &SocketAddr) {
        debug!(
            "SBC NAT Traversal: Appending rport/received for source socket {}",
            src_addr
        );
    }
}
