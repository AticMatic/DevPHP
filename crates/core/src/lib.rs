pub mod binaries;
pub mod config;
pub mod error;
pub mod services;
pub mod system;

pub use config::AppConfig;
pub use error::DevPhpError;
pub use services::service_registry::ServiceRegistry;

/// Initialize the tracing subscriber for structured logging.
pub fn init_logging() {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("devphp_core=info"));

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .init();
}
