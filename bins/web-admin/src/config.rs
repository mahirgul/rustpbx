use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ServerConfig {
    pub http_bind_addr: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct DatabaseConfig {
    pub db_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: ServerConfig {
                http_bind_addr: "0.0.0.0:8088".to_string(),
            },
            database: DatabaseConfig {
                db_path: "sqlite:data/rustpbx.db".to_string(),
            },
        }
    }
}
