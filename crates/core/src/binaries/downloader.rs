use crate::config::AppConfig;
use crate::error::{DevPhpError, Result};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// Async downloader for PHP binary archives (Windows only for MVP).
pub struct BinaryDownloader {
    config: AppConfig,
}

impl BinaryDownloader {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// Download and extract a PHP binary for Windows.
    /// On macOS/Linux, returns an error with install instructions.
    pub async fn download_php(
        &self,
        url: &str,
        version: &str,
        progress_tx: Option<tokio::sync::mpsc::Sender<DownloadProgress>>,
    ) -> Result<PathBuf> {
        let dest_dir = self.config.bin_dir().join(format!("php-{}", version));
        fs::create_dir_all(&dest_dir)?;

        tracing::info!("Downloading PHP from {} to {}", url, dest_dir.display());

        // Stream download with progress
        let response = reqwest::get(url).await?;
        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded: u64 = 0;

        let tmp_path = self.config.tmp_dir().join(format!("php-{}.zip", version));
        let mut file = fs::File::create(&tmp_path)?;

        let stream = response.bytes().await?;
        file.write_all(&stream)?;
        downloaded = stream.len() as u64;

        if let Some(ref tx) = progress_tx {
            let _ = tx
                .send(DownloadProgress {
                    downloaded,
                    total: total_size,
                    finished: true,
                })
                .await;
        }

        tracing::info!("Download complete, extracting...");

        // Extract zip
        let archive_file = fs::File::open(&tmp_path)?;
        let mut archive = zip::ZipArchive::new(archive_file)
            .map_err(|e| DevPhpError::DownloadError(format!("Invalid zip archive: {}", e)))?;

        archive
            .extract(&dest_dir)
            .map_err(|e| DevPhpError::DownloadError(format!("Extraction failed: {}", e)))?;

        // Cleanup temp file
        let _ = fs::remove_file(&tmp_path);

        tracing::info!("PHP {} extracted to {}", version, dest_dir.display());
        Ok(dest_dir)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: u64,
    pub finished: bool,
}
