use config::{Config, ConfigError, File};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub media_storage: MediaStorageConfig,
    pub service: ServiceConfig,
    pub auth: AuthConfig,
    pub cors: CorsConfig,
    pub logging: LoggingConfig,
    pub pagination: PaginationConfig,
    pub cleanup: CleanupConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout_seconds: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MediaStorageConfig {
    pub base_path: PathBuf,
    pub max_file_size: u64,
    pub temp_path: PathBuf,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServiceConfig {
    pub name: String,
    pub description: String,
    pub version: String,
    pub media_store_type: String,
    pub public_url_base: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AuthConfig {
    pub require_auth: bool,
    pub jwt_secret: String,
    pub basic_auth_username: String,
    pub basic_auth_password: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PaginationConfig {
    pub default_limit: u32,
    pub max_limit: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CleanupConfig {
    pub temp_file_retention_hours: u64,
    pub orphaned_object_retention_days: u64,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(File::with_name("config"))
            .build()?;

        config.try_deserialize()
    }

    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(File::with_name(path))
            .build()?;

        config.try_deserialize()
    }
} 