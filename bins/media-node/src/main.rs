mod config;
mod grpc;
mod recorder;
mod rtp;

use config::Config;
use grpc::MediaControlService;
use pbx_proto::media::media_control_server::MediaControlServer;
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cfg = Config::default();
    let grpc_addr = cfg.server.grpc_bind_addr.parse()?;

    info!("Starting RustPBX Media Node process...");
    info!("gRPC MediaControl Server listening on {}", grpc_addr);

    let service = MediaControlService::new(cfg.rtp.bind_ip, cfg.rtp.port_range_start);

    Server::builder()
        .add_service(MediaControlServer::new(service))
        .serve(grpc_addr)
        .await?;

    Ok(())
}
