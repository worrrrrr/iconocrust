//! Typed configuration for Iconocrust Core.

use std::env;
use std::fmt;
use std::net::{IpAddr, SocketAddr};

use thiserror::Error;

/// Configuration errors.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid port: {0}")]
    InvalidPort(String),
    #[error("invalid host: {0}")]
    InvalidHost(String),
    #[error("data directory not writable: {0}")]
    DataDirNotWritable(String),
}

/// Typed configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub data_dir: String,
}

impl Config {
    /// Load configuration from environment variables with safe defaults.
    pub fn from_env() -> Result<Self, ConfigError> {
        let host = env::var("ICONOCRUST_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

        // Validate host is a valid IP address
        if host.parse::<std::net::IpAddr>().is_err() {
            return Err(ConfigError::InvalidHost(host));
        }

        // Loopback-only by default: non-loopback requires explicit opt-in
        // (enforced at bind time in main.rs)

        let port: u16 = env::var("ICONOCRUST_PORT")
            .unwrap_or_else(|_| "7432".to_string())
            .parse()
            .map_err(|e| ConfigError::InvalidPort(format!("{}", e)))?;

        let data_dir = env::var("ICONOCRUST_DATA_DIR").unwrap_or_else(|_| default_data_dir());

        Ok(Self {
            host,
            port,
            data_dir,
        })
    }

    /// Inject configuration for tests.
    pub fn new(host: &str, port: u16, data_dir: &str) -> Result<Self, ConfigError> {
        if host.parse::<std::net::IpAddr>().is_err() {
            return Err(ConfigError::InvalidHost(host.to_string()));
        }
        Ok(Self {
            host: host.to_string(),
            port,
            data_dir: data_dir.to_string(),
        })
    }

    /// Parsed IP address of the configured host.
    fn ip(&self) -> IpAddr {
        self.host
            .parse()
            .expect("host is validated on construction")
    }

    /// Socket address to bind.
    ///
    /// Works for IPv4 and IPv6 hosts; bracketing is handled by `SocketAddr`.
    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.ip(), self.port)
    }

    /// Check if the host is loopback.
    pub fn is_loopback(&self) -> bool {
        self.ip().is_loopback()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 7432,
            data_dir: default_data_dir(),
        }
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "host={}, port={}, data_dir={}",
            self.host, self.port, self.data_dir
        )
    }
}

fn default_data_dir() -> String {
    // Platform-appropriate: use env var or fallback to current directory
    env::var("LOCALAPPDATA")
        .or_else(|_| env::var("XDG_DATA_HOME"))
        .unwrap_or_else(|_| ".".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_loopback() {
        let cfg = Config::default();
        assert!(cfg.is_loopback());
        assert_eq!(cfg.port, 7432);
    }

    #[test]
    fn invalid_host_fails() {
        let result = Config::new("not-an-ip", 7432, ".");
        assert!(result.is_err());
    }

    #[test]
    fn valid_config_parses() {
        let cfg = Config::new("127.0.0.1", 8080, "/tmp/data").unwrap();
        assert_eq!(cfg.port, 8080);
        assert_eq!(cfg.data_dir, "/tmp/data");
    }

    #[test]
    fn ipv6_loopback_socket_addr() {
        let cfg = Config::new("::1", 7432, ".").unwrap();
        assert!(cfg.is_loopback());
        assert_eq!(
            cfg.socket_addr(),
            "[::1]:7432".parse::<SocketAddr>().unwrap()
        );
    }

    #[test]
    fn ipv4_loopback_socket_addr() {
        let cfg = Config::new("127.0.0.1", 8080, ".").unwrap();
        assert!(cfg.is_loopback());
        assert_eq!(
            cfg.socket_addr(),
            "127.0.0.1:8080".parse::<SocketAddr>().unwrap()
        );
    }

    #[test]
    fn non_loopback_is_detected() {
        let cfg = Config::new("0.0.0.0", 7432, ".").unwrap();
        assert!(!cfg.is_loopback());
    }

    #[test]
    fn config_display_does_not_leak_secrets() {
        let cfg = Config::default();
        let s = format!("{}", cfg);
        assert!(!s.contains("password"));
    }
}
