#![allow(dead_code)]

use figment::{
    Figment,
    providers::{Env, Format, Yaml},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub redis: RedisSettings,
    pub jwt: JwtSettings,
    pub smtp: SmtpSettings,
}

#[derive(Debug, Deserialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub mtls_ca_cert: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize)]
pub struct RedisSettings {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct JwtSettings {
    pub secret: String,
    pub access_token_duration_minutes: u64,
    pub refresh_token_duration_days: u64,
}

#[derive(Debug, Deserialize)]
pub struct SmtpSettings {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    pub from: String,
}

pub fn get_config() -> Result<Settings, figment::Error> {
    Figment::new()
        .merge(Yaml::file("config.yaml"))
        .merge(Env::prefixed("AUTH_SERVER__").split("__"))
        .extract()
}
