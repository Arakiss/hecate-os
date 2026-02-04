//! ISO download module

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub struct IsoDownloader;

impl IsoDownloader {
    pub async fn download_ubuntu(version: &str, output_path: &Path) -> Result<()> {
        let url = match version {
            "24.04" | "latest" => "https://releases.ubuntu.com/24.04.2/ubuntu-24.04.2-desktop-amd64.iso",
            "22.04" => "https://releases.ubuntu.com/22.04.5/ubuntu-22.04.5-desktop-amd64.iso",
            "server" => "https://releases.ubuntu.com/24.04.2/ubuntu-24.04.2-live-server-amd64.iso",
            _ => return Err(anyhow::anyhow!("Unsupported Ubuntu version: {}. Use '24.04', '22.04', or 'server'", version)),
        };
        
        println!("📥 Downloading Ubuntu {} ISO...", version);
        println!("   From: {}", url);
        
        // Create HTTP client with redirect support
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(10))
            .timeout(std::time::Duration::from_secs(3600))
            .build()?;
        
        let response = client
            .get(url)
            .send()
            .await
            .context("Failed to start download")?;
        
        // Check if the response is successful
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Download failed with status: {}", response.status()));
        }
        
        let total_size = response
            .content_length()
            .ok_or_else(|| anyhow::anyhow!("Failed to get content length"))?;
        
        // Create progress bar
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-")
        );
        
        // Download with progress
        let mut file = File::create(output_path)
            .context("Failed to create output file")?;
        
        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();
        
        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Error during download")?;
            file.write_all(&chunk)?;
            
            let new = std::cmp::min(downloaded + (chunk.len() as u64), total_size);
            downloaded = new;
            pb.set_position(new);
        }
        
        pb.finish_with_message("Download complete");
        
        // Verify download
        let file_size = std::fs::metadata(output_path)?.len();
        if file_size != total_size {
            std::fs::remove_file(output_path)?;
            return Err(anyhow::anyhow!("Download corrupted: size mismatch"));
        }
        
        Ok(())
    }
    
    pub fn cleanup(path: &Path) -> Result<()> {
        if path.exists() {
            std::fs::remove_file(path)
                .context("Failed to cleanup ISO file")?;
        }
        Ok(())
    }
}