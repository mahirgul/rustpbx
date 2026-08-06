use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Config {
    pub server: ServerConfig,
    pub sip_engine: SipEngineConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ServerConfig {
    pub wss_bind_addr: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct SipEngineConfig {
    pub sip_udp_target: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: ServerConfig {
                wss_bind_addr: "0.0.0.0:8089".to_string(),
            },
            sip_engine: SipEngineConfig {
                sip_udp_target: "127.0.0.1:5060".to_string(),
            },
        }
    }
}
