use crate::config::AppConfig;
use crate::error::{DevPhpError, Result};
use crate::services::php_service::{PhpService, ServiceStatus};
use crate::system::process_manager::ProcessManager;
use parking_lot::RwLock;
use std::sync::Arc;

/// Central orchestrator that owns the ProcessManager and all services.
pub struct ServiceRegistry {
    process_manager: ProcessManager,
    php_service: RwLock<PhpService>,
    config: Arc<RwLock<AppConfig>>,
}

impl ServiceRegistry {
    pub fn new(config: AppConfig) -> Self {
        let pm = ProcessManager::new();
        let php = PhpService::new(pm.clone(), config.clone());
        Self {
            process_manager: pm,
            php_service: RwLock::new(php),
            config: Arc::new(RwLock::new(config)),
        }
    }

    /// Start a service by name.
    pub fn start(&self, name: &str) -> Result<ServiceStatus> {
        match name {
            "php" => self.php_service.write().start(),
            _ => Err(DevPhpError::ServiceNotFound(name.to_string())),
        }
    }

    /// Stop a service by name.
    pub fn stop(&self, name: &str) -> Result<ServiceStatus> {
        match name {
            "php" => self.php_service.write().stop(),
            _ => Err(DevPhpError::ServiceNotFound(name.to_string())),
        }
    }

    /// Get status of a service by name.
    pub fn status(&self, name: &str) -> Result<ServiceStatus> {
        match name {
            "php" => Ok(self.php_service.read().status()),
            _ => Err(DevPhpError::ServiceNotFound(name.to_string())),
        }
    }

    /// Get status of all registered services.
    pub fn status_all(&self) -> Vec<ServiceStatus> {
        vec![self.php_service.read().status()]
    }

    /// Stop all services and kill all processes (used on app exit).
    pub fn stop_all(&self) {
        let _ = self.php_service.write().stop();
        self.process_manager.kill_all();
    }

    /// Run watchdog health check on all processes.
    /// Returns names of services that died unexpectedly.
    pub fn check_health(&self) -> Vec<String> {
        self.process_manager.check_health()
    }

    /// Get the PHP log file path.
    pub fn php_log_path(&self) -> std::path::PathBuf {
        self.php_service.read().log_path()
    }
}
