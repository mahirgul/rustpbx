use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Config {
    pub sip: SipConfig,
    pub api: ApiConfig,
    pub media_node: MediaNodeConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct SipConfig {
    pub udp_bind_addr: String,
    pub domain: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ApiConfig {
    pub http_bind_addr: String,
    pub secret_key: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct MediaNodeConfig {
    pub grpc_target_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            sip: SipConfig {
                udp_bind_addr: "0.0.0.0:5060".to_string(),
                domain: "pbx.local".to_string(),
            },
            api: ApiConfig {
                http_bind_addr: "0.0.0.0:8080".to_string(),
                secret_key: "change-me-in-production".to_string(),
            },
            media_node: MediaNodeConfig {
                grpc_target_url: "http://127.0.0.1:50051".to_string(),
            },
        }
    }
}
