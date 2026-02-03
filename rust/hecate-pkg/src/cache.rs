//! Package cache management
//!
//! Handles download cache, parallel downloads, and delta updates

use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use tokio::fs;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};

use crate::Package;

/// Package cache for downloaded packages
pub struct PackageCache {
    cache_dir: PathBuf,
    max_cache_size: u64,
}

impl PackageCache {
    /// Create a new package cache
    pub fn new(cache_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(cache_dir)
            .context("Failed to create cache directory")?;
        
        Ok(Self {
            cache_dir: cache_dir.to_path_buf(),
            max_cache_size: 10 * 1024 * 1024 * 1024, // 10GB default
        })
    }

    /// Get the cache path for a package
    pub fn get_package_path(&self, package: &Package) -> PathBuf {
        let filename = format!("{}-{}.pkg.tar.zst", package.name, package.version);
        self.cache_dir.join(filename)
    }

    /// Get delta package path
    pub fn get_delta_path(&self, package: &Package, from_version: &str) -> PathBuf {
        let filename = format!("{}-{}-to-{}.delta.zst", 
            package.name, from_version, package.version);
        self.cache_dir.join("deltas").join(filename)
    }

    /// Clean old cached packages
    pub async fn clean(&self, keep_count: usize) -> Result<u64> {
        let mut entries = Vec::new();
        let mut total_freed = 0u64;

        // Collect all cache entries with metadata
        let mut dir = fs::read_dir(&self.cache_dir).await?;
        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            if path.extension() == Some(std::ffi::OsStr::new("zst")) {
                let metadata = entry.metadata().await?;
                let modified = metadata.modified()?;
                entries.push((path, modified, metadata.len()));
            }
        }

        // Sort by modification time (newest first)
        entries.sort_by_key(|(_, modified, _)| std::cmp::Reverse(*modified));

        // Keep only the specified number of recent packages
        for (path, _, size) in entries.iter().skip(keep_count) {
            fs::remove_file(path).await?;
            total_freed += size;
        }

        Ok(total_freed)
    }

    /// Clean packages older than specified days
    pub async fn clean_old(&self, days: u64) -> Result<u64> {
        use std::time::{SystemTime, Duration};
        
        let cutoff = SystemTime::now() - Duration::from_secs(days * 24 * 3600);
        let mut total_freed = 0u64;

        let mut dir = fs::read_dir(&self.cache_dir).await?;
        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            let metadata = entry.metadata().await?;
            
            if let Ok(modified) = metadata.modified() {
                if modified < cutoff {
                    total_freed += metadata.len();
                    fs::remove_file(path).await?;
                }
            }
        }

        Ok(total_freed)
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> Result<CacheStats> {
        let mut total_size = 0u64;
        let mut package_count = 0usize;
        let mut delta_count = 0usize;

        let mut dir = fs::read_dir(&self.cache_dir).await?;
        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            let metadata = entry.metadata().await?;
            
            if path.extension() == Some(std::ffi::OsStr::new("zst")) {
                total_size += metadata.len();
                
                if path.to_string_lossy().contains(".delta.") {
                    delta_count += 1;
                } else {
                    package_count += 1;
                }
            }
        }

        Ok(CacheStats {
            total_size,
            package_count,
            delta_count,
            cache_dir: self.cache_dir.clone(),
        })
    }

    /// Verify cache integrity
    pub async fn verify_integrity(&self) -> Result<Vec<String>> {
        let mut corrupted = Vec::new();

        let mut dir = fs::read_dir(&self.cache_dir).await?;
        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            
            if path.extension() == Some(std::ffi::OsStr::new("zst")) {
                // Try to decompress header to verify integrity
                if let Ok(file) = std::fs::File::open(&path) {
                    let decoder = zstd::Decoder::new(file);
                    if decoder.is_err() {
                        corrupted.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        Ok(corrupted)
    }

    /// Prune cache to stay under size limit
    pub async fn prune_to_size(&self, max_size: u64) -> Result<u64> {
        let stats = self.get_stats().await?;
        
        if stats.total_size <= max_size {
            return Ok(0);
        }

        let mut entries = Vec::new();
        let mut dir = fs::read_dir(&self.cache_dir).await?;
        
        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            if path.extension() == Some(std::ffi::OsStr::new("zst")) {
                let metadata = entry.metadata().await?;
                let modified = metadata.modified()?;
                entries.push((path, modified, metadata.len()));
            }
        }

        // Sort by modification time (oldest first)
        entries.sort_by_key(|(_, modified, _)| *modified);

        let target_freed = stats.total_size - max_size;
        let mut total_freed = 0u64;

        for (path, _, size) in entries {
            if total_freed >= target_freed {
                break;
            }
            
            fs::remove_file(path).await?;
            total_freed += size;
        }

        Ok(total_freed)
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_size: u64,
    pub package_count: usize,
    pub delta_count: usize,
    pub cache_dir: PathBuf,
}

/// Parallel download manager
pub struct DownloadManager {
    client: reqwest::Client,
    parallel_downloads: usize,
    progress: MultiProgress,
}

impl DownloadManager {
    /// Create a new download manager
    pub fn new(parallel_downloads: usize) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("hecate-pkg/0.1.0")
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            parallel_downloads,
            progress: MultiProgress::new(),
        }
    }

    /// Download multiple packages in parallel
    pub async fn download_packages(
        &self,
        downloads: Vec<(String, PathBuf, u64)>, // (url, destination, expected_size)
    ) -> Result<Vec<Result<PathBuf>>> {
        use futures::stream;

        let tasks = downloads.into_iter().map(|(url, dest, size)| {
            self.download_file(url, dest, size)
        });

        let results: Vec<Result<PathBuf>> = stream::iter(tasks)
            .buffer_unordered(self.parallel_downloads)
            .collect()
            .await;

        Ok(results)
    }

    /// Download a single file with progress
    async fn download_file(
        &self,
        url: String,
        destination: PathBuf,
        expected_size: u64,
    ) -> Result<PathBuf> {
        // Create progress bar
        let pb = self.progress.add(ProgressBar::new(expected_size));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} {msg}")?
                .progress_chars("##-"),
        );
        
        let filename = destination.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        pb.set_message(format!("Downloading {}", filename));

        // Start download
        let response = self.client.get(&url).send().await
            .context("Failed to start download")?;
        
        let status = response.status();
        if !status.is_success() {
            return Err(anyhow::anyhow!("Download failed with status: {}", status));
        }

        // Get actual size if available
        if let Some(len) = response.content_length() {
            pb.set_length(len);
        }

        // Create destination file
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        let mut file = fs::File::create(&destination).await?;
        let mut stream = response.bytes_stream();

        // Stream to file
        use tokio::io::AsyncWriteExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Failed to read chunk")?;
            file.write_all(&chunk).await
                .context("Failed to write chunk")?;
            pb.inc(chunk.len() as u64);
        }

        pb.finish_with_message(format!("Downloaded {}", filename));

        Ok(destination)
    }

    /// Download with resume support
    pub async fn download_with_resume(
        &self,
        url: &str,
        destination: &Path,
        expected_size: u64,
    ) -> Result<PathBuf> {
        // Check if partial download exists
        let mut resume_from = 0u64;
        if destination.exists() {
            let metadata = fs::metadata(destination).await?;
            resume_from = metadata.len();
            
            if resume_from >= expected_size {
                // Already fully downloaded
                return Ok(destination.to_path_buf());
            }
        }

        // Create request with range header for resume
        let mut request = self.client.get(url);
        if resume_from > 0 {
            request = request.header("Range", format!("bytes={}-", resume_from));
        }

        let response = request.send().await?;
        
        if !response.status().is_success() && response.status() != 206 {
            return Err(anyhow::anyhow!("Download failed with status: {}", response.status()));
        }

        // Create progress bar
        let pb = self.progress.add(ProgressBar::new(expected_size));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} {msg}")?
                .progress_chars("##-"),
        );
        pb.set_position(resume_from);
        pb.set_message(format!("Resuming {}", destination.file_name().unwrap_or_default().to_string_lossy()));

        // Open file for appending
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(resume_from > 0)
            .write(true)
            .open(destination)
            .await?;

        // Stream to file
        let mut stream = response.bytes_stream();
        use tokio::io::AsyncWriteExt;
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            pb.inc(chunk.len() as u64);
        }

        pb.finish_with_message(format!("Completed {}", destination.file_name().unwrap_or_default().to_string_lossy()));

        Ok(destination.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_cache_creation() {
        let dir = tempdir().unwrap();
        let cache = PackageCache::new(dir.path()).unwrap();
        
        let stats = cache.get_stats().await.unwrap();
        assert_eq!(stats.package_count, 0);
        assert_eq!(stats.total_size, 0);
    }

    #[tokio::test]
    async fn test_cache_paths() {
        let dir = tempdir().unwrap();
        let cache = PackageCache::new(dir.path()).unwrap();
        
        let package = Package {
            name: "test".to_string(),
            version: semver::Version::parse("1.0.0").unwrap(),
            description: String::new(),
            author: String::new(),
            license: String::new(),
            homepage: None,
            repository: None,
            dependencies: Vec::new(),
            conflicts: Vec::new(),
            provides: Vec::new(),
            replaces: Vec::new(),
            categories: Vec::new(),
            keywords: Vec::new(),
            architecture: crate::Architecture::X86_64,
            size_bytes: 0,
            installed_size_bytes: 0,
            checksum: crate::PackageChecksum {
                sha256: String::new(),
                blake3: String::new(),
            },
            signature: None,
            build_date: chrono::Utc::now(),
        };
        
        let path = cache.get_package_path(&package);
        assert!(path.to_string_lossy().contains("test-1.0.0.pkg.tar.zst"));
        
        let delta_path = cache.get_delta_path(&package, "0.9.0");
        assert!(delta_path.to_string_lossy().contains("test-0.9.0-to-1.0.0.delta.zst"));
    }
}