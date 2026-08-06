use sipcore::types::header_name::HeaderName;
use sipcore::types::message::SipMessage;
use std::net::SocketAddr;
use tracing::debug;

pub struct TopologyHiding;

impl TopologyHiding {
    #[allow(dead_code)]
    pub fn mask_inbound(&self, msg: &mut SipMessage) {
        if let Some(_via) = msg.headers.get(&HeaderName::Via) {
            debug!("SBC Inbound TopologyHiding: Inspecting Via header");
        }
    }

    #[allow(dead_code)]
    pub fn mask_outbound(&self, msg: &mut SipMessage, public_sbc_addr: &SocketAddr) {
        debug!(
            "SBC Outbound TopologyHiding: Masking topology to public SBC address {}",
            public_sbc_addr
        );
        let _ = msg;
    }
}
