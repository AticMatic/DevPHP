// Prevents additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use devphp_core::config::AppConfig;
use devphp_core::services::php_service::ServiceStatus;
use devphp_core::services::service_registry::ServiceRegistry;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::io::AsyncBufReadExt;

struct AppState {
    registry: Arc<ServiceRegistry>,
}

#[tauri::command]
async fn start_services(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<ServiceStatus, String> {
    let registry = state.registry.clone();
    let status = registry.start("php").map_err(|e| e.to_string())?;

    // Start log streaming in background
    let log_path = registry.php_log_path();
    let app_handle = app.clone();
    tokio::spawn(async move {
        stream_log_file(app_handle, log_path).await;
    });

    // Start watchdog in background
    let registry_wd = registry.clone();
    let app_wd = app.clone();
    tokio::spawn(async move {
        run_watchdog(app_wd, registry_wd).await;
    });

    Ok(status)
}

#[tauri::command]
async fn stop_services(state: State<'_, AppState>) -> Result<ServiceStatus, String> {
    state.registry.stop("php").map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_service_status(state: State<'_, AppState>) -> Result<Vec<ServiceStatus>, String> {
    Ok(state.registry.status_all())
}

async fn stream_log_file(app: AppHandle, log_path: std::path::PathBuf) {
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    loop {
        let file = match tokio::fs::File::open(&log_path).await {
            Ok(f) => f,
            Err(_) => {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                continue;
            }
        };

        // Seek to end to only get new lines
        let reader = tokio::io::BufReader::new(file);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            let _ = app.emit("log-line", &line);
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    }
}

async fn run_watchdog(app: AppHandle, registry: Arc<ServiceRegistry>) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        let dead = registry.check_health();
        for name in &dead {
            let _ = app.emit("service-died", name);
        }
        let statuses = registry.status_all();
        let _ = app.emit("service-status", &statuses);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    devphp_core::init_logging();

    let config = AppConfig::load().unwrap_or_default();
    if let Err(e) = config.ensure_dirs() {
        eprintln!("Failed to create DevPHP directories: {}", e);
    }
    let _ = config.save();

    let registry = Arc::new(ServiceRegistry::new(config));
    let registry_cleanup = registry.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState { registry })
        .invoke_handler(tauri::generate_handler![
            start_services,
            stop_services,
            get_service_status
        ])
        .on_window_event(move |_window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                registry_cleanup.stop_all();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running DevPHP");
}
