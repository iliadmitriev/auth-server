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
    #[serde(default = "default_log_format")]
    pub log_format: String,
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

pub fn load_settings() -> Result<Settings, Box<figment::Error>> {
    Figment::new()
        .merge(Yaml::file("config.yaml"))
        .merge(Env::prefixed("AUTH_SERVER__").split("__"))
        .extract()
        .map_err(Box::new)
}

fn default_log_format() -> String {
    "pretty".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use figment::providers::Yaml;

    fn full_yaml() -> &'static str {
        r#"
server:
  host: "127.0.0.1"
  port: 8080
  mtls_ca_cert: "/path/to/ca.crt"
  log_format: "json"
database:
  url: "postgres://localhost/test"
  max_connections: 10
redis:
  url: "redis://localhost"
jwt:
  secret: "supersecret"
  access_token_duration_minutes: 15
  refresh_token_duration_days: 7
smtp:
  host: "smtp.example.com"
  port: 587
  username: "user"
  password: "pass"
  from: "noreply@example.com"
"#
    }

    #[test]
    fn full_yaml_deserializes_all_fields() {
        let settings: Settings = Figment::new()
            .merge(Yaml::string(full_yaml()))
            .extract()
            .unwrap();

        assert_eq!(settings.server.host, "127.0.0.1");
        assert_eq!(settings.server.port, 8080);
        assert_eq!(
            settings.server.mtls_ca_cert,
            Some("/path/to/ca.crt".to_string())
        );
        assert_eq!(settings.server.log_format, "json");
        assert_eq!(settings.database.url, "postgres://localhost/test");
        assert_eq!(settings.database.max_connections, 10);
        assert_eq!(settings.redis.url, "redis://localhost");
        assert_eq!(settings.jwt.secret, "supersecret");
        assert_eq!(settings.jwt.access_token_duration_minutes, 15);
        assert_eq!(settings.jwt.refresh_token_duration_days, 7);
        assert_eq!(settings.smtp.host, "smtp.example.com");
        assert_eq!(settings.smtp.port, 587);
        assert_eq!(settings.smtp.username, Some("user".to_string()));
        assert_eq!(settings.smtp.password, Some("pass".to_string()));
        assert_eq!(settings.smtp.from, "noreply@example.com");
    }

    fn minimal_yaml() -> &'static str {
        r#"
server:
  host: "0.0.0.0"
  port: 3000
database:
  url: "postgres://localhost/test"
  max_connections: 5
redis:
  url: "redis://localhost"
jwt:
  secret: "key"
  access_token_duration_minutes: 30
  refresh_token_duration_days: 30
smtp:
  host: "smtp.example.com"
  port: 25
  from: "noreply@example.com"
"#
    }

    #[test]
    fn optional_fields_get_defaults() {
        let settings: Settings = Figment::new()
            .merge(Yaml::string(minimal_yaml()))
            .extract()
            .unwrap();

        assert_eq!(settings.server.mtls_ca_cert, None);
        assert_eq!(settings.server.log_format, "pretty");
        assert_eq!(settings.smtp.username, None);
        assert_eq!(settings.smtp.password, None);
    }
}
