use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Config {
    pub server: ServerConfig,
    pub rtp: RtpConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ServerConfig {
    pub grpc_bind_addr: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct RtpConfig {
    pub bind_ip: String,
    pub port_range_start: u16,
    pub port_range_end: u16,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct StorageConfig {
    pub recording_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: ServerConfig {
                grpc_bind_addr: "127.0.0.1:50051".to_string(),
            },
            rtp: RtpConfig {
                bind_ip: "0.0.0.0".to_string(),
                port_range_start: 10000,
                port_range_end: 20000,
            },
            storage: StorageConfig {
                recording_dir: "/var/lib/rustpbx/recordings".to_string(),
            },
        }
    }
}
