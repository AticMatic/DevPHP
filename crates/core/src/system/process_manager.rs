use crate::error::{DevPhpError, Result};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::Instant;

/// Tracked child process with metadata.
struct TrackedProcess {
    child: Child,
    started_at: Instant,
    log_path: Option<PathBuf>,
}

/// Manages spawned child processes with graceful shutdown and watchdog support.
#[derive(Clone)]
pub struct ProcessManager {
    processes: Arc<RwLock<HashMap<String, TrackedProcess>>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Spawn a new process and track it by name.
    /// stdout/stderr are redirected to `log_path` if provided.
    pub fn spawn(
        &self,
        name: &str,
        program: &str,
        args: &[&str],
        log_path: Option<PathBuf>,
    ) -> Result<u32> {
        // Check if already running
        if self.is_running(name) {
            return Err(DevPhpError::ProcessError(format!(
                "Process '{}' is already running",
                name
            )));
        }

        let mut cmd = Command::new(program);
        cmd.args(args);

        // Redirect output to log file if specified
        if let Some(ref path) = log_path {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let stdout_file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path)?;
            let stderr_file = stdout_file.try_clone()?;
            cmd.stdout(Stdio::from(stdout_file));
            cmd.stderr(Stdio::from(stderr_file));
        } else {
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());
        }

        // Prevent the child from inheriting the parent's stdin
        cmd.stdin(Stdio::null());

        let child = cmd
            .spawn()
            .map_err(|e| DevPhpError::ProcessError(format!("Failed to spawn '{}': {}", name, e)))?;

        let pid = child.id();
        tracing::info!("Spawned process '{}' with PID {}", name, pid);

        let tracked = TrackedProcess {
            child,
            started_at: Instant::now(),
            log_path,
        };

        self.processes.write().insert(name.to_string(), tracked);
        Ok(pid)
    }

    /// Graceful shutdown: SIGTERM → wait 3s → SIGKILL (Unix) or TerminateProcess (Windows).
    pub fn kill(&self, name: &str) -> Result<()> {
        let mut processes = self.processes.write();
        if let Some(mut tracked) = processes.remove(name) {
            tracing::info!("Stopping process '{}'", name);

            #[cfg(unix)]
            {
                // Send SIGTERM first
                let pid = tracked.child.id();
                unsafe {
                    libc::kill(pid as i32, libc::SIGTERM);
                }
                // Wait up to 3 seconds for graceful exit
                let deadline = Instant::now() + std::time::Duration::from_secs(3);
                loop {
                    match tracked.child.try_wait() {
                        Ok(Some(_status)) => {
                            tracing::info!("Process '{}' exited gracefully", name);
                            return Ok(());
                        }
                        Ok(None) => {
                            if Instant::now() >= deadline {
                                break;
                            }
                            std::thread::sleep(std::time::Duration::from_millis(100));
                        }
                        Err(e) => {
                            tracing::warn!("Error waiting for process '{}': {}", name, e);
                            break;
                        }
                    }
                }
                // Force kill if still alive
                tracing::warn!("Process '{}' did not exit gracefully, sending SIGKILL", name);
                let _ = tracked.child.kill();
                let _ = tracked.child.wait();
            }

            #[cfg(windows)]
            {
                // Windows: TerminateProcess via kill()
                let _ = tracked.child.kill();
                let _ = tracked.child.wait();
            }

            tracing::info!("Process '{}' terminated", name);
            Ok(())
        } else {
            Err(DevPhpError::ProcessError(format!(
                "No running process named '{}'",
                name
            )))
        }
    }

    /// Kill all tracked processes (used on app exit).
    pub fn kill_all(&self) {
        let names: Vec<String> = self.processes.read().keys().cloned().collect();
        for name in names {
            if let Err(e) = self.kill(&name) {
                tracing::error!("Failed to kill '{}': {}", name, e);
            }
        }
    }

    /// Check if a process is alive (read-only, non-blocking).
    pub fn is_running(&self, name: &str) -> bool {
        let processes = self.processes.read();
        processes.contains_key(name)
    }

    /// Get the PID of a tracked process.
    pub fn get_pid(&self, name: &str) -> Option<u32> {
        let processes = self.processes.read();
        processes.get(name).map(|p| p.child.id())
    }

    /// Get the uptime in seconds.
    pub fn uptime_secs(&self, name: &str) -> Option<u64> {
        let processes = self.processes.read();
        processes
            .get(name)
            .map(|p| p.started_at.elapsed().as_secs())
    }

    /// Watchdog: check all processes and remove any that have exited.
    /// Returns list of process names that were found dead.
    pub fn check_health(&self) -> Vec<String> {
        let mut dead = Vec::new();
        let mut processes = self.processes.write();

        processes.retain(|name, tracked| {
            match tracked.child.try_wait() {
                Ok(Some(status)) => {
                    tracing::warn!(
                        "Process '{}' exited unexpectedly with status: {}",
                        name,
                        status
                    );
                    dead.push(name.clone());
                    false // remove from map
                }
                Ok(None) => true,  // still running
                Err(e) => {
                    tracing::error!("Error checking process '{}': {}", name, e);
                    dead.push(name.clone());
                    false
                }
            }
        });

        dead
    }

    /// Get the log path for a process.
    pub fn log_path(&self, name: &str) -> Option<PathBuf> {
        let processes = self.processes.read();
        processes.get(name).and_then(|p| p.log_path.clone())
    }
}

impl Drop for ProcessManager {
    fn drop(&mut self) {
        self.kill_all();
    }
}
