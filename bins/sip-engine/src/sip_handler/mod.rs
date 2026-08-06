pub mod invite;
pub mod options;
pub mod register;
pub mod response;
pub mod utils;

use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

use sipcore::types::message::{SipMessage, StartLine};
use sipcore::types::method::Method;
use sipcore::types::status_code::StatusCode;
use sipstack::UdpTransport;

use crate::config::Config;
use crate::db::DbStore;

use invite::{handle_bye, handle_cancel, handle_invite};
use options::handle_options;
use register::handle_register;
use response::send_simple_response;

pub async fn handle_incoming_sip_message(
    msg: SipMessage,
    src: SocketAddr,
    transport: &Arc<UdpTransport>,
    db: &DbStore,
    cfg: &Config,
) {
    match &msg.start_line {
        StartLine::Request(req) => {
            info!("Received SIP Request {} from {}", req.method, src);

            match req.method {
                Method::Register => {
                    handle_register(msg, src, transport, db, cfg).await;
                }
                Method::Options => {
                    handle_options(msg, src, transport).await;
                }
                Method::Invite => {
                    handle_invite(msg, src, transport, db, cfg).await;
                }
                Method::Bye => {
                    handle_bye(msg, src, transport).await;
                }
                Method::Cancel => {
                    handle_cancel(msg, src, transport).await;
                }
                _ => {
                    send_simple_response(msg, StatusCode::new(200).unwrap(), src, transport).await;
                }
            }
        }
        StartLine::Response(resp) => {
            info!(
                "Received SIP Response {} from {}",
                resp.status_code.code(),
                src
            );
        }
    }
}
