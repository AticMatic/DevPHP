use crate::config::AppConfig;
use crate::error::{DevPhpError, Result};
use crate::system::port_manager::PortManager;
use crate::system::process_manager::ProcessManager;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const PHP_PROCESS_NAME: &str = "php";

#[derive(Debug, Clone, Serialize)]
pub struct ServiceStatus {
    pub name: String,
    pub running: bool,
    pub port: Option<u16>,
    pub pid: Option<u32>,
    pub uptime_secs: Option<u64>,
    pub php_version: Option<String>,
}

pub struct PhpService {
    process_manager: ProcessManager,
    config: AppConfig,
    active_port: Option<u16>,
    detected_version: Option<String>,
}

impl PhpService {
    pub fn new(process_manager: ProcessManager, config: AppConfig) -> Self {
        Self {
            process_manager,
            config,
            active_port: None,
            detected_version: None,
        }
    }

    /// Detect PHP version by running `php -v` and parsing output.
    pub fn detect_php_version(php_path: &str) -> Option<String> {
        let output = Command::new(php_path).arg("-v").output().ok()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Parse "PHP 8.3.14 (cli) ..."
        stdout
            .lines()
            .next()
            .and_then(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 && parts[0] == "PHP" {
                    Some(parts[1].to_string())
                } else {
                    None
                }
            })
    }

    /// Resolve the PHP binary path using the resolution order:
    /// 1. ~/.devphp/bin/php-{version}/php
    /// 2. System PHP (which php / where php)
    fn resolve_php_binary(&self) -> Result<String> {
        // 1. Check managed binary
        let managed_path = self
            .config
            .bin_dir()
            .join(format!("php-{}", self.config.php_version))
            .join(if cfg!(windows) { "php.exe" } else { "php" });

        if managed_path.exists() {
            tracing::info!("Using managed PHP at {}", managed_path.display());
            return Ok(managed_path.to_string_lossy().to_string());
        }

        // 2. Check system PHP
        let which_cmd = if cfg!(windows) { "where" } else { "which" };
        let output = Command::new(which_cmd)
            .arg("php")
            .output()
            .map_err(|e| DevPhpError::PhpNotFound(format!("Failed to search for PHP: {}", e)))?;

        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                tracing::info!("Using system PHP at {}", path);
                return Ok(path);
            }
        }

        // 3. Not found
        let hint = if cfg!(target_os = "macos") {
            "Install PHP via Homebrew: brew install php"
        } else if cfg!(windows) {
            "Download PHP from https://windows.php.net/download/"
        } else {
            "Install PHP via your system package manager"
        };

        Err(DevPhpError::PhpNotFound(hint.to_string()))
    }

    /// Generate a minimal php.ini.
    fn generate_php_ini(&self) -> Result<PathBuf> {
        let ini_path = self.config.config_dir().join("php.ini");
        let ini_content = format!(
            r#"; DevPHP generated php.ini
; Generated at: {}

[PHP]
display_errors = On
error_reporting = E_ALL
log_errors = On
error_log = {}
date.timezone = UTC
memory_limit = 256M
max_execution_time = 30

[opcache]
opcache.enable = 0
"#,
            chrono_placeholder(),
            self.config
                .logs_dir()
                .join("php_errors.log")
                .to_string_lossy()
        );

        fs::write(&ini_path, ini_content)?;
        tracing::info!("Generated php.ini at {}", ini_path.display());
        Ok(ini_path)
    }

    /// Write the default phpinfo() index.php to docroot (if empty).
    fn ensure_default_docroot(&self) -> Result<()> {
        let index_path = self.config.www_dir().join("index.php");
        if !index_path.exists() {
            fs::create_dir_all(self.config.www_dir())?;
            let content = r#"<?php
// DevPHP Default Page
phpinfo();
"#;
            fs::write(&index_path, content)?;
            tracing::info!("Created default index.php at {}", index_path.display());
        }

        // Create health check endpoint
        let health_path = self.config.www_dir().join("devphp-health.php");
        if !health_path.exists() {
            fs::write(&health_path, r#"<?php echo "OK";"#)?;
            tracing::info!(
                "Created health check at {}",
                health_path.display()
            );
        }

        Ok(())
    }

    /// Start the PHP built-in development server.
    pub fn start(&mut self) -> Result<ServiceStatus> {
        if self.process_manager.is_running(PHP_PROCESS_NAME) {
            return Err(DevPhpError::ProcessError(
                "PHP service is already running".to_string(),
            ));
        }

        // Ensure directory structure
        self.config.ensure_dirs()?;

        // Resolve PHP binary
        let php_path = self.resolve_php_binary()?;

        // Detect version
        self.detected_version = Self::detect_php_version(&php_path);
        tracing::info!(
            "PHP version: {}",
            self.detected_version.as_deref().unwrap_or("unknown")
        );

        // Find free port (prefer last used)
        let port = PortManager::find_free_port(self.config.preferred_port(), 20)?;
        self.active_port = Some(port);

        // Save last used port
        let _ = self.config.set_last_port(port);

        // Generate php.ini
        let ini_path = self.generate_php_ini()?;

        // Ensure docroot has content
        self.ensure_default_docroot()?;

        // Build args
        let bind_addr = format!("0.0.0.0:{}", port);
        let ini_str = ini_path.to_string_lossy().to_string();
        let docroot_str = self.config.www_dir().to_string_lossy().to_string();
        let log_path = self.config.logs_dir().join("php.log");

        let args = vec![
            "-S",
            &bind_addr,
            "-c",
            &ini_str,
            "-t",
            &docroot_str,
        ];

        // Spawn PHP server
        self.process_manager
            .spawn(PHP_PROCESS_NAME, &php_path, &args, Some(log_path))?;

        tracing::info!(
            "PHP server started on http://localhost:{} serving {}",
            port,
            docroot_str
        );

        Ok(self.status())
    }

    /// Stop the PHP server.
    pub fn stop(&mut self) -> Result<ServiceStatus> {
        self.process_manager.kill(PHP_PROCESS_NAME)?;
        let port = self.active_port.take();
        tracing::info!("PHP server stopped (was on port {:?})", port);
        Ok(self.status())
    }

    /// Get current service status.
    pub fn status(&self) -> ServiceStatus {
        let running = self.process_manager.is_running(PHP_PROCESS_NAME);
        ServiceStatus {
            name: "PHP Development Server".to_string(),
            running,
            port: if running { self.active_port } else { None },
            pid: self.process_manager.get_pid(PHP_PROCESS_NAME),
            uptime_secs: self.process_manager.uptime_secs(PHP_PROCESS_NAME),
            php_version: self.detected_version.clone(),
        }
    }

    /// Get the log file path.
    pub fn log_path(&self) -> PathBuf {
        self.config.logs_dir().join("php.log")
    }

    /// Get a reference to the process manager.
    pub fn process_manager(&self) -> &ProcessManager {
        &self.process_manager
    }
}

fn chrono_placeholder() -> String {
    // Simple timestamp without chrono dependency
    format!("{:?}", std::time::SystemTime::now())
}
