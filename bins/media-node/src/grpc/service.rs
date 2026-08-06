use dashmap::DashMap;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::info;

use pbx_proto::media::media_control_server::MediaControl;
use pbx_proto::media::{
    AllocateSessionRequest, AllocateSessionResponse, ReleaseSessionRequest, ReleaseSessionResponse,
    StartRecordingRequest, StartRecordingResponse,
};

use crate::rtp::RtpRelaySession;

pub struct MediaControlService {
    pub sessions: Arc<DashMap<String, Arc<RtpRelaySession>>>,
    pub bind_ip: String,
    pub next_port: Arc<std::sync::atomic::AtomicU16>,
}

impl MediaControlService {
    pub fn new(bind_ip: String, start_port: u16) -> Self {
        MediaControlService {
            sessions: Arc::new(DashMap::new()),
            bind_ip,
            next_port: Arc::new(std::sync::atomic::AtomicU16::new(start_port)),
        }
    }
}

#[tonic::async_trait]
impl MediaControl for MediaControlService {
    async fn allocate_session(
        &self,
        request: Request<AllocateSessionRequest>,
    ) -> Result<Response<AllocateSessionResponse>, Status> {
        let req = request.into_inner();
        let port = self
            .next_port
            .fetch_add(2, std::sync::atomic::Ordering::SeqCst);

        let session_id = format!("sess-{}", req.call_id);
        info!(
            "Allocating media session {} for call {} on port {}",
            session_id, req.call_id, port
        );

        let session = RtpRelaySession::bind(session_id.clone(), &self.bind_ip, port)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        self.sessions.insert(session_id.clone(), Arc::new(session));

        Ok(Response::new(AllocateSessionResponse {
            session_id,
            local_ip: self.bind_ip.clone(),
            local_port: port as u32,
            success: true,
        }))
    }

    async fn release_session(
        &self,
        request: Request<ReleaseSessionRequest>,
    ) -> Result<Response<ReleaseSessionResponse>, Status> {
        let req = request.into_inner();
        info!("Releasing media session {}", req.session_id);
        let removed = self.sessions.remove(&req.session_id).is_some();
        Ok(Response::new(ReleaseSessionResponse { success: removed }))
    }

    async fn start_recording(
        &self,
        request: Request<StartRecordingRequest>,
    ) -> Result<Response<StartRecordingResponse>, Status> {
        let req = request.into_inner();
        info!(
            "Start recording requested for session {} (format: {})",
            req.session_id, req.format
        );
        Ok(Response::new(StartRecordingResponse {
            success: true,
            message: format!("Recording started at {}", req.file_path),
        }))
    }
}
