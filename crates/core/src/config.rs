use crate::error::{DevPhpError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub base_dir: PathBuf,
    pub php_port: u16,
    pub php_version: String,
    pub docroot: PathBuf,
    pub last_used_port: Option<u16>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let base = Self::default_base_dir();
        Self {
            docroot: base.join("www"),
            base_dir: base,
            php_port: 8080,
            php_version: String::from("system"),
            last_used_port: None,
        }
    }
}

impl AppConfig {
    /// Platform-appropriate base directory: ~/.devphp or %APPDATA%/devphp
    fn default_base_dir() -> PathBuf {
        if cfg!(windows) {
            dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("C:\\devphp"))
                .join("devphp")
        } else {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join(".devphp")
        }
    }

    /// Path to the config.json file.
    fn config_path(&self) -> PathBuf {
        self.base_dir.join("config.json")
    }

    /// Load config from disk, or return default if not found.
    pub fn load() -> Result<Self> {
        let default_base = Self::default_base_dir();
        let config_path = default_base.join("config.json");

        if config_path.exists() {
            let data = fs::read_to_string(&config_path)?;
            let config: AppConfig = serde_json::from_str(&data)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Persist current config to disk.
    pub fn save(&self) -> Result<()> {
        let config_path = self.config_path();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, data)?;
        tracing::info!("Config saved to {}", config_path.display());
        Ok(())
    }

    /// Create the full directory tree under base_dir.
    pub fn ensure_dirs(&self) -> Result<()> {
        let dirs = ["bin", "config", "sites", "logs", "data", "tmp", "www"];
        for dir in &dirs {
            let path = self.base_dir.join(dir);
            fs::create_dir_all(&path)?;
            tracing::debug!("Ensured directory: {}", path.display());
        }
        Ok(())
    }

    /// Get the preferred starting port (last used, or configured default).
    pub fn preferred_port(&self) -> u16 {
        self.last_used_port.unwrap_or(self.php_port)
    }

    /// Update and persist the last used port.
    pub fn set_last_port(&mut self, port: u16) -> Result<()> {
        self.last_used_port = Some(port);
        self.save()
    }

    // Directory accessors
    pub fn bin_dir(&self) -> PathBuf {
        self.base_dir.join("bin")
    }
    pub fn config_dir(&self) -> PathBuf {
        self.base_dir.join("config")
    }
    pub fn logs_dir(&self) -> PathBuf {
        self.base_dir.join("logs")
    }
    pub fn www_dir(&self) -> PathBuf {
        self.docroot.clone()
    }
    pub fn tmp_dir(&self) -> PathBuf {
        self.base_dir.join("tmp")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_ensure_dirs_creates_all() {
        let tmp = tempdir().unwrap();
        let config = AppConfig {
            base_dir: tmp.path().to_path_buf(),
            docroot: tmp.path().join("www"),
            ..Default::default()
        };
        config.ensure_dirs().unwrap();

        for dir in &["bin", "config", "sites", "logs", "data", "tmp", "www"] {
            assert!(tmp.path().join(dir).is_dir(), "Missing dir: {}", dir);
        }
    }

    #[test]
    fn test_save_and_load() {
        let tmp = tempdir().unwrap();
        let mut config = AppConfig {
            base_dir: tmp.path().to_path_buf(),
            docroot: tmp.path().join("www"),
            php_port: 9090,
            ..Default::default()
        };
        config.save().unwrap();

        // Manually load from the same path
        let data = fs::read_to_string(tmp.path().join("config.json")).unwrap();
        let loaded: AppConfig = serde_json::from_str(&data).unwrap();
        assert_eq!(loaded.php_port, 9090);
    }
}
