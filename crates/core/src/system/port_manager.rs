use crate::error::{DevPhpError, Result};
use std::net::TcpListener;

/// Manages port availability checks and auto-fallback.
pub struct PortManager;

impl PortManager {
    /// Check if a specific port is available by attempting to bind.
    pub fn is_port_available(port: u16) -> bool {
        TcpListener::bind(("127.0.0.1", port)).is_ok()
    }

    /// Find a free port starting from `preferred`, trying up to `max_attempts` ports.
    /// Returns the first available port or an error if all are occupied.
    pub fn find_free_port(preferred: u16, max_attempts: u16) -> Result<u16> {
        let end = preferred.saturating_add(max_attempts);
        for port in preferred..end {
            if Self::is_port_available(port) {
                tracing::info!("Port {} is available", port);
                return Ok(port);
            }
            tracing::debug!("Port {} is occupied, trying next", port);
        }
        Err(DevPhpError::PortRangeExhausted(
            preferred,
            preferred,
            end.saturating_sub(1),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn test_occupied_port_detected() {
        // Bind a port so it's occupied
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        assert!(!PortManager::is_port_available(port));
    }

    #[test]
    fn test_find_free_port_succeeds() {
        let port = PortManager::find_free_port(49152, 20).unwrap();
        assert!(port >= 49152);
    }
}
